using System;
using System.Collections.Generic;
using System.Linq;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Foundation;
using Windows.System.UserProfile;

namespace SeokdokSettings;

/// <summary>Builders for the cards of a settings page, after Windows 11's own Settings.</summary>
internal static class Ui
{
    /// <summary>Korean when Korean is the first language of the Windows account, else English.</summary>
    public static readonly bool Korean =
        GlobalizationPreferences.Languages.FirstOrDefault()?.StartsWith("ko", StringComparison.OrdinalIgnoreCase) ?? false;

    /// <summary>The label in the language of the window.</summary>
    public static string L(string korean, string english) => Korean ? korean : english;

    public static Style Theme(string key) => (Style)Application.Current.Resources[key];

    public static TextBlock Text(string text, string style = "BodyTextBlockStyle")
    {
        return new TextBlock { Text = text, Style = Theme(style), TextWrapping = TextWrapping.Wrap };
    }

    public static TextBlock Caption(string text) => Text(text, "SettingDescription");

    public static TextBlock Section(string text)
    {
        var t = Text(text, "BodyStrongTextBlockStyle");
        t.Margin = new Thickness(1, 20, 0, 4);
        return t;
    }

    /// <summary>
    /// A card: what the setting does, its effect, and its control. The control
    /// goes under the words when the window is too narrow for both.
    /// </summary>
    public static Border Card(string header, string? description, string? example, FrameworkElement? control, string? glyph = null)
    {
        var words = new StackPanel { Spacing = 2, VerticalAlignment = VerticalAlignment.Center };
        words.Children.Add(Text(header));
        if (!string.IsNullOrEmpty(description)) words.Children.Add(Caption(description));
        if (!string.IsNullOrEmpty(example)) words.Children.Add(Text(example, "SettingExample"));

        var grid = new Grid();
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        if (glyph is not null)
        {
            var icon = new FontIcon { Glyph = glyph, FontSize = 20, VerticalAlignment = VerticalAlignment.Center, Margin = new Thickness(0, 0, 16, 0) };
            grid.Children.Add(icon);
            Grid.SetRowSpan(icon, 2);
        }
        Grid.SetColumn(words, 1);
        grid.Children.Add(words);
        if (control is not null)
        {
            control.VerticalAlignment = VerticalAlignment.Center;
            grid.Children.Add(control);
            Grid.SetColumn(control, 2);
        }

        var card = new Border { Style = Theme("SettingCard"), Child = grid };
        if (control is not null)
        {
            card.SizeChanged += (_, e) =>
            {
                var stacked = e.NewSize.Width < 620;
                grid.RowSpacing = stacked ? 8 : 0;
                Grid.SetRow(control, stacked ? 1 : 0);
                Grid.SetColumn(control, stacked ? 1 : 2);
                control.HorizontalAlignment = stacked ? HorizontalAlignment.Left : HorizontalAlignment.Right;
                control.Margin = new Thickness(stacked ? 0 : 16, 0, 0, 0);
            };
        }
        return card;
    }

    public static ToggleSwitch Toggle(bool on, Action<bool> changed)
    {
        var t = new ToggleSwitch { IsOn = on, OnContent = L("켜짐", "On"), OffContent = L("꺼짐", "Off"), MinWidth = 0 };
        t.Toggled += (_, _) => changed(t.IsOn);
        return t;
    }

    public static ComboBox Choice(IEnumerable<string> labels, int selected, Action<int> changed)
    {
        var c = new ComboBox { MinWidth = 220, MaxWidth = 420 };
        foreach (var label in labels) c.Items.Add(new ComboBoxItem { Content = new TextBlock { Text = label, TextWrapping = TextWrapping.Wrap } });
        c.SelectedIndex = selected;
        c.SelectionChanged += (_, _) => { if (c.SelectedIndex >= 0) changed(c.SelectedIndex); };
        return c;
    }

    /// <summary>A collapsed group for what few people change.</summary>
    public static Expander More(string header, string description, IEnumerable<UIElement> cards, bool expanded, Action<bool> remember)
    {
        var words = new StackPanel { Spacing = 2, Margin = new Thickness(0, 12, 0, 12) };
        words.Children.Add(Text(header));
        words.Children.Add(Caption(description));
        var inside = new StackPanel { Spacing = 4 };
        foreach (var c in cards) inside.Children.Add(c);
        var e = new Expander
        {
            Header = words,
            Content = inside,
            IsExpanded = expanded,
            HorizontalAlignment = HorizontalAlignment.Stretch,
            HorizontalContentAlignment = HorizontalAlignment.Stretch,
            Margin = new Thickness(0, 4, 0, 0),
        };
        e.Expanding += (_, _) => remember(true);
        e.Collapsed += (_, _) => remember(false);
        return e;
    }

    public static TextBlock PageTitle(string text)
    {
        var t = Text(text, "TitleTextBlockStyle");
        t.Margin = new Thickness(0, 0, 0, 4);
        return t;
    }
}

/// <summary>Lays its children out in rows that wrap.</summary>
internal sealed partial class WrapPanel : Panel
{
    const double Gap = 6;

    protected override Size MeasureOverride(Size available)
    {
        double x = 0, y = 0, row = 0, widest = 0;
        foreach (var child in Children)
        {
            child.Measure(available);
            var s = child.DesiredSize;
            if (x > 0 && x + s.Width > available.Width)
            {
                y += row + Gap;
                x = row = 0;
            }
            x += s.Width + Gap;
            row = Math.Max(row, s.Height);
            widest = Math.Max(widest, x - Gap);
        }
        return new Size(widest, y + row);
    }

    protected override Size ArrangeOverride(Size final)
    {
        double y = 0;
        var row = new List<UIElement>();
        double x = 0, height = 0;
        void Place()
        {
            double at = 0;
            foreach (var child in row)
            {
                var s = child.DesiredSize;
                child.Arrange(new Rect(at, y + (height - s.Height) / 2, s.Width, s.Height));
                at += s.Width + Gap;
            }
            y += height + Gap;
            row.Clear();
            x = height = 0;
        }
        foreach (var child in Children)
        {
            var s = child.DesiredSize;
            if (x > 0 && x + s.Width > final.Width) Place();
            row.Add(child);
            x += s.Width + Gap;
            height = Math.Max(height, s.Height);
        }
        Place();
        return final;
    }
}
