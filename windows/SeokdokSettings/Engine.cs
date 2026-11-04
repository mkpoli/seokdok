using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Threading;

namespace SeokdokSettings;

/// <summary>What a sample came out as, typed through the real session.</summary>
internal sealed record Typed(string Text, IReadOnlyList<string> Candidates, int Total);

/// <summary>What a preview shows: a sample before and after a change.</summary>
internal sealed record Outcome(string? Problem, string Keys, Typed? Before, Typed? After);

/// <summary>
/// The Rust session on a thread of its own, so a change writes the file and
/// reads the result in order, and a dictionary load never holds up the window.
/// </summary>
internal sealed class Engine
{
    public static readonly Engine Shared = new();

    readonly BlockingCollection<Action> work = new();
    IntPtr ime;
    bool broken;

    Engine()
    {
        var thread = new Thread(Run) { IsBackground = true, Name = "engine" };
        thread.Start();
    }

    void Run()
    {
        var dict = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "dict"));
        try
        {
            ime = Native.seokdok_new(dict);
        }
        catch (Exception e) when (e is DllNotFoundException or EntryPointNotFoundException)
        {
            broken = true;
        }
        foreach (var job in work.GetConsumingEnumerable()) job();
    }

    bool WaitReady()
    {
        for (var i = 0; i < 600 && !broken; i++)
        {
            if (Native.seokdok_ready()) return true;
            Thread.Sleep(50);
        }
        return false;
    }

    /// <summary>
    /// Types `keys` as they are, then presses `trigger` and Enter. Letters in
    /// upper case are the Shift layer; a space is Shift+Space.
    /// </summary>
    Typed Type(string keys, int mode, int trigger)
    {
        Native.seokdok_reset(ime);
        Native.seokdok_set_mode(ime, mode);
        Press(keys);
        Native.seokdok_key(ime, trigger, trigger == Native.SpaceKey ? ' ' : 0u, false);
        // Keys the session leaves to the application come out as themselves (Latin mode).
        var eaten = Native.seokdok_key(ime, Native.Enter, 0, false);
        var commit = Native.Borrowed(Native.seokdok_take_commit(ime));
        if (!eaten && commit.Length == 0 && mode == 2) commit = keys;
        Native.seokdok_reset(ime);
        var (list, total) = Candidates(keys, mode);
        return new Typed(commit, list, total);
    }

    void Press(string keys)
    {
        foreach (var k in keys)
        {
            if (k == ' ') Native.seokdok_key(ime, Native.SpaceKey, ' ', true);
            else Native.seokdok_key(ime, Native.Letter, k, char.IsAsciiLetterUpper(k));
        }
    }

    /// <summary>The list the candidate window shows for `keys`.</summary>
    (IReadOnlyList<string> List, int Total) Candidates(string keys, int mode)
    {
        var list = new List<string>();
        // A phrase of several words lists one word at a time.
        if (mode != 0 || keys.Contains(' ')) return (list, 0);
        Native.seokdok_reset(ime);
        Native.seokdok_set_mode(ime, mode);
        Press(keys);
        Native.seokdok_key(ime, Native.Convert, 0, false);
        if (Native.seokdok_popup_len(ime) == 0) Native.seokdok_key(ime, Native.Convert, 0, false);
        var n = (nuint)Native.seokdok_popup_len(ime);
        for (nuint i = 0; i < n; i++) list.Add(Native.Borrowed(Native.seokdok_popup_item(ime, i, IntPtr.Zero)));
        Native.seokdok_popup_info(ime, out _, out _, out var total, out _);
        Native.seokdok_reset(ime);
        return (list, (int)total);
    }

    static int ModeOf(string value) => value switch { "hangul" => 1, "latin" => 2, _ => 0 };

    /// <summary>
    /// Changes the file by `patch` and types the sample before and after, then
    /// calls `done`. A null patch only types; a null sample only changes the file.
    /// </summary>
    public void Change(string? patch, Sample? sample, Action<Outcome> done)
    {
        work.Add(() =>
        {
            Typed? before = null;
            var ready = !broken && Native.seokdok_ready();
            if (sample is not null && patch is not null && ready)
            {
                Native.seokdok_sync(ime);
                before = TypeSample(sample);
            }
            string? problem = null;
            if (patch is not null)
            {
                try { problem = Native.UpdateConfig(patch); }
                catch (DllNotFoundException e) { problem = e.Message; }
            }
            Typed? after = null;
            if (problem is null && sample is not null && !broken && WaitReady())
            {
                Native.seokdok_sync(ime);
                after = TypeSample(sample);
            }
            done(new Outcome(problem, sample?.Keys ?? "", before, after));
        });
    }

    Typed TypeSample(Sample s) =>
        Type(s.Keys, s.FollowStartMode ? StartMode() : 0, s.Trigger);

    /// <summary>Read from the file: on this thread a change has reached it before the window hears.</summary>
    static int StartMode() =>
        ModeOf(System.Text.Json.Nodes.JsonNode.Parse(Native.ConfigJson())?["input"]?["default_mode"]?.GetValue<string>() ?? "");

    /// <summary>Read custom.json again after it was saved.</summary>
    public void ReloadCustom() => work.Add(() =>
    {
        if (!broken) Native.seokdok_reload(ime);
    });
}

/// <summary>What to type for a preview: the keys, and which key converts them.</summary>
internal sealed record Sample(string Hangul, string Keys, int Trigger = Native.Convert, bool FollowStartMode = false);
