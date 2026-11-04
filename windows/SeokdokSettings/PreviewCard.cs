using System.Collections.Generic;
using System.Linq;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace SeokdokSettings;

/// <summary>
/// The sample a setting changes, typed through the real engine: what was
/// typed, what it gave before the change, what it gives now, and the candidates.
/// </summary>
internal sealed class PreviewCard
{
    public readonly Border View;
    readonly TextBlock title = Ui.Caption("");
    readonly TextBlock typed = Ui.Text("", "PreviewTyped");
    readonly TextBlock before = Ui.Text("", "PreviewBefore");
    readonly TextBlock arrow = Ui.Text("→", "PreviewTyped");
    readonly TextBlock result = Ui.Text("", "PreviewResult");
    readonly StackPanel candidates = new() { Spacing = 8 };
    readonly DispatcherQueue queue = DispatcherQueue.GetForCurrentThread();
    int shown;

    public PreviewCard()
    {
        typed.TextWrapping = TextWrapping.NoWrap;
        var line = new WrapPanel();
        foreach (var part in new FrameworkElement[] { typed, arrow, before, result })
        {
            part.VerticalAlignment = VerticalAlignment.Center;
            line.Children.Add(part);
        }
        var inside = new StackPanel { Spacing = 6 };
        inside.Children.Add(title);
        inside.Children.Add(line);
        inside.Children.Add(candidates);
        View = new Border { Style = Ui.Theme("PreviewPanel"), Child = inside };
    }

    /// <summary>Types the sample without changing anything.</summary>
    public void Show(string heading, Sample sample) => Apply(heading, sample, null, null);

    /// <summary>Writes `patch`, then shows what it did to the sample; `after` runs on the window's thread.</summary>
    public void Apply(string heading, Sample sample, string? patch, System.Action<string?>? after)
    {
        var ticket = ++shown;
        title.Text = heading;
        typed.Text = sample.Hangul;
        Engine.Shared.Change(patch, sample, outcome => queue.TryEnqueue(() =>
        {
            after?.Invoke(outcome.Problem);
            if (ticket != shown) return;
            if (outcome.Problem is not null) return;
            if (outcome.After is null)
            {
                result.Text = Ui.L("辭典을 讀는 中", "Loading the dictionaries");
                before.Text = "";
                return;
            }
            var changed = outcome.Before is not null && outcome.Before.Text != outcome.After.Text;
            before.Text = changed ? outcome.Before!.Text : "";
            result.Text = outcome.After.Text;
            FillCandidates(outcome.After);
        }));
    }

    void FillCandidates(Typed typedResult)
    {
        var list = typedResult.Candidates;
        candidates.Children.Clear();
        candidates.Visibility = list.Count == 0 ? Visibility.Collapsed : Visibility.Visible;
        if (list.Count == 0) return;
        candidates.Orientation = Settings.Str("input.candidate_layout") == "horizontal" ? Orientation.Horizontal : Orientation.Vertical;
        candidates.Spacing = candidates.Orientation == Orientation.Horizontal ? 14 : 1;
        var n = 0;
        foreach (var c in list.Take(3))
        {
            n++;
            candidates.Children.Add(new TextBlock { Text = n + "  " + c, Style = Ui.Theme("PreviewCandidate") });
        }
        if (typedResult.Total > n)
            candidates.Children.Add(new TextBlock { Text = Ui.L($"全部 {typedResult.Total} 個", $"{typedResult.Total} in all"), Style = Ui.Theme("PreviewCandidate") });
    }
}
