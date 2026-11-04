using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace SeokdokSettings;

/// <summary>examples.tsv: what a setting does, as the engine writes it. A test in crates/seokdok-ffi checks every row.</summary>
internal static class Examples
{
    public sealed record Row(string Id, string Patch, string Keys, string Text);

    static readonly Dictionary<string, Row> Rows = Load();

    static Dictionary<string, Row> Load()
    {
        var path = Path.Combine(AppContext.BaseDirectory, "examples.tsv");
        var rows = new Dictionary<string, Row>();
        if (!File.Exists(path)) return rows;
        foreach (var line in File.ReadAllLines(path))
        {
            if (line.Length == 0 || line[0] == '#') continue;
            var f = line.Split('\t');
            if (f.Length == 4) rows[f[0]] = new Row(f[0], f[1], f[2], f[3]);
        }
        return rows;
    }

    public static string Text(string id) => Rows.TryGetValue(id, out var r) ? r.Text : "";
    public static string Keys(string id) => Rows.TryGetValue(id, out var r) ? r.Keys : "";

    /// <summary>"봤다 → 見ㅆ다 · 見ㅏㅆ다 · 見았다": a Hangul word and what each choice makes of it.</summary>
    public static string Line(string hangul, params string[] ids) =>
        hangul + " → " + string.Join(" · ", ids.Select(Text));
}
