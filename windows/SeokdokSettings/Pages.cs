using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json.Nodes;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using static SeokdokSettings.Ui;

namespace SeokdokSettings;

/// <summary>A page of settings: fixed content above a scrolling column, rebuilt when the file changes.</summary>
internal abstract class Page
{
    public abstract string Title { get; }
    public abstract string Glyph { get; }
    public abstract string Id { get; }

    protected StackPanel Column = new();
    protected PreviewCard? Preview;
    public Action<string>? Report;
    public Action? Rebuild;

    /// <summary>Which collapsed groups are open, for as long as the window lives.</summary>
    protected static readonly HashSet<string> Open = new();

    /// <summary>Writes still on their way to the file.</summary>
    public static int Writing { get; private set; }

    /// <summary>Whether the page shows what config.toml holds, and follows it.</summary>
    public virtual bool FollowsFile => true;

    ScrollViewer? scroller;
    Grid? holder;
    StackPanel? head;
    bool narrow;

    /// <summary>In a narrow window the pane button takes the corner, and the margins shrink.</summary>
    public void SetNarrow(bool value)
    {
        narrow = value;
        if (head is not null) head.Margin = value ? new Thickness(16, 8, 16, 4) : new Thickness(36, 8, 36, 4);
        if (head?.Children[0] is FrameworkElement title) title.Margin = new Thickness(value ? 40 : 0, 0, 0, 4);
        if (holder is not null) holder.Padding = value ? new Thickness(16, 0, 16, 24) : new Thickness(36, 0, 36, 36);
    }

    protected abstract void Build();

    /// <summary>The page is about to be shown.</summary>
    public virtual void Enter() { }

    /// <summary>The page is going away: nothing typed may stay unsaved.</summary>
    public virtual void Leave() { }

    public FrameworkElement Render()
    {
        var offset = scroller?.VerticalOffset ?? 0;
        // A new column each time: the old one still belongs to the page on show.
        Column = new StackPanel { Spacing = 4, MaxWidth = 1000 };
        Preview = null;
        Build();

        var root = new Grid { RowSpacing = 0 };
        root.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        root.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
        head = new StackPanel { MaxWidth = 1072 };
        head.Children.Add(PageTitle(Title));
        if (Preview is not null) head.Children.Add(Preview.View);
        root.Children.Add(head);
        holder = new Grid();
        holder.Children.Add(Column);
        SetNarrow(narrow);
        var scroll = new ScrollViewer { Content = holder, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled };
        Grid.SetRow(scroll, 1);
        root.Children.Add(scroll);
        scroll.Loaded += (_, _) => scroll.ChangeView(null, offset, null, true);
        scroller = scroll;
        return root;
    }

    protected void Add(UIElement e) => Column.Children.Add(e);

    /// <summary>Writes a change. With a preview, the engine shows what it did to the sample.</summary>
    protected void Change(string heading, Sample? sample, string patch, bool rebuild = false)
    {
        Writing++;
        void After(string? problem)
        {
            Writing--;
            if (problem is not null) Report?.Invoke(problem);
            Settings.Reload();
            Refresh();
            if (rebuild || problem is not null) Rebuild?.Invoke();
        }
        if (Preview is not null && sample is not null) Preview.Apply(heading, sample, patch, After);
        else Engine.Shared.Change(patch, null, o => Program.Queue?.TryEnqueue(() => After(o.Problem)));
    }

    /// <summary>What depends on another setting follows it.</summary>
    protected virtual void Refresh() { }
}

internal sealed class ConversionPage : Page
{
    public override string Id => "conversion";
    public override string Title => L("變換", "Conversion");
    public override string Glyph => "\uE8D2";

    static readonly Sample Sentence = new("선생님께서 공부하셨다", Examples.Keys("script.sino_only"));
    static readonly Sample Saw = new("봤다", Examples.Keys("ending.drop"));
    static readonly Sample Be = new("있다", Examples.Keys("itda.yu"));
    static readonly Sample Eat = new("먹으면", Examples.Keys("final.on"));
    static readonly Sample Do = new("하고", Examples.Keys("ha.modern"));
    static readonly Sample Studied = new("공부했다", Examples.Keys("ha.both"));

    Border? carry;
    ToggleSwitch? carrySwitch;

    protected override void Build()
    {
        Preview = new PreviewCard();
        Preview.Show(L("미리 보기", "Preview"), Sentence);

        var scripts = Product.Scripts.ToArray();
        string Label(string script) => script switch
        {
            "seokdok" => L("釋讀: 固有語까지 漢字로", "釋讀: native words in Hanja too"),
            "sino_only" => L("國漢文混用: 漢字語만 漢字로", "國漢文混用: Sino-Korean words only"),
            _ => L("韓㐎 專用: 變換하지 않음", "Hangul only: no conversion"),
        };
        Add(Card(
            L("韓㐎을 무엇으로 變換하나", "What your Hangul becomes"),
            Product.HasKun
                ? L("Space를 押면 이렇게 變합니다. 作業 표시줄 아이콘의 釋 混 한입니다.", "Space turns what you typed into this. The taskbar icon shows it as 釋, 混 or 한.")
                : L("Space를 누르면 이렇게 變합니다. 作業 표시줄 아이콘의 混 한입니다.", "Space turns what you typed into this. The taskbar icon shows it as 混 or 한."),
            "선생님께서 공부하셨다 →\n" + string.Join("\n", scripts.Select(s => Examples.Text("script." + s))),
            Choice(
                scripts.Select(Label).ToArray(),
                Array.IndexOf(scripts, Settings.Str("conversion.script")),
                i => Change(L("韓㐎을 무엇으로 變換하나", "What your Hangul becomes"), Sentence, Settings.Patch("conversion.script", scripts[i]))),
            "\uE8D2"));

        Add(More(
            L("表記 더 보기", "More spelling options"),
            Product.HasKun
                ? L("語尾, 받침, 있다와 하다를 적는 法", "Endings, final consonants, 있다 and 하다")
                : L("하를 적는 法, 候補 數", "How 하 is written, how many candidates"),
            Spelling(), Open.Contains(Id), on => { if (on) Open.Add(Id); else Open.Remove(Id); }));
        Refresh();
    }

    IEnumerable<UIElement> Spelling()
    {
        foreach (var card in KunSpelling()) yield return card;

        // hada_kun and old_ha: both decide how 하 comes out. The one with 爲 stays
        // out of a product that hides it.
        var kun = Settings.Bool("conversion.hada_kun");
        var old = Settings.Bool("conversion.old_ha");
        var ha = L("하를 적는 法", "How 하 is written");
        yield return Product.Hides("hada_kun")
            ? Card(
                ha,
                L("먼저 나오는 表記입니다. 다른 表記도 候補에 남습니다.", "Listed first. The others stay among the candidates."),
                Examples.Line("하고", "ha.modern", "ha.old"),
                Choice(
                    new[] { L("그대로 하고", "As it is: 하고"), L("옛 表記로 ᄒᆞ고", "Old spelling: ᄒᆞ고") },
                    old ? 1 : 0,
                    i => Change(ha, Do, Settings.Patch("conversion.old_ha", i == 1))))
            : Card(
                ha,
                L("먼저 나오는 表記입니다. 다른 表記도 候補에 남습니다.", "Listed first. The others stay among the candidates."),
                Examples.Line("하고", "ha.modern", "ha.kun", "ha.old") + "  ·  " + Examples.Line("공부했다", "ha.both"),
                Choice(
                    new[] { L("그대로 하고", "As it is: 하고"), L("漢字로 爲고, 成는", "In Hanja: 爲고, 成는"), L("옛 表記로 ᄒᆞ고", "Old spelling: ᄒᆞ고"), L("옛 表記로, 했다만 爲ㅆ다", "Old spelling, and 爲ㅆ다 for 했다") },
                    old ? (kun ? 3 : 2) : (kun ? 1 : 0),
                    i => Change(ha, i == 3 ? Studied : Do, Settings.Patch(("conversion.hada_kun", i is 1 or 3), ("conversion.old_ha", i >= 2)))));

        var count = L("候補 數", "How many candidates");
        var box = new NumberBox { Minimum = 1, Maximum = 99, Value = Settings.Int("conversion.max_candidates"), SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Compact, MinWidth = 120 };
        box.ValueChanged += (_, e) =>
        {
            if (!double.IsNaN(e.NewValue)) Change(count, Product.HasKun ? Saw : Sentence, Settings.Patch("conversion.max_candidates", (int)e.NewValue));
        };
        yield return Card(count, L("句節 全體를 變換한 候補의 上限입니다. 낱말 候補는 그대로여서 目錄의 맨 끝만 조금 달라집니다. 미리 보기에 全部 몇 個인지 나옵니다.", "A limit on conversions of the whole phrase. Word candidates are not affected, so only the end of the list changes a little. The preview shows the total."), null, box);
    }

    /// <summary>How 訓讀 is spelled; none of this exists in a product without it.</summary>
    IEnumerable<UIElement> KunSpelling()
    {
        if (!Product.HasKun) yield break;

        // style and fragment: the leftover vowel only exists in the short ending.
        var morpheme = Settings.Str("conversion.style") == "morpheme";
        var letter = Settings.Str("conversion.fragment") == "letter";
        var ending = L("漢字 뒤의 語尾", "The ending after a Hanja verb");
        yield return Card(
            ending,
            L("動詞를 漢字로 적을 때 뒤에 남기는 韓㐎", "How much of the verb stays in Hangul"),
            Examples.Line("봤다", "ending.drop", "ending.letter", "ending.whole"),
            Choice(
                new[] { L("줄여서 見ㅆ다", "Short: 見ㅆ다"), L("남는 母音도 字母로 見ㅏㅆ다", "With the vowel as a letter: 見ㅏㅆ다"), L("語尾 그대로 見았다", "The whole ending: 見았다") },
                morpheme ? 2 : letter ? 1 : 0,
                i => Change(ending, Saw, i switch
                {
                    0 => Settings.Patch(("conversion.style", "surface"), ("conversion.fragment", "absorb")),
                    1 => Settings.Patch(("conversion.style", "surface"), ("conversion.fragment", "letter")),
                    _ => Settings.Patch("conversion.style", "morpheme"),
                })));

        string[] itdas = { "有", "在", "在ㅆ" };
        var itda = L("있다를 적는 字", "The Hanja for 있다");
        yield return Card(
            itda,
            L("먼저 나오는 表記입니다. 다른 表記도 候補에 남습니다.", "Listed first. The others stay among the candidates."),
            Examples.Line("있다", "itda.yu", "itda.jae", "itda.jae_coda"),
            Choice(new[] { "有다", "在다", "在ㅆ다" }, Array.IndexOf(itdas, Settings.Str("conversion.itda")),
                i => Change(itda, Be, Settings.Patch("conversion.itda", itdas[i]))));

        var final = L("動詞의 받침을 漢字 뒤에 적기", "Write the verb's final consonant after the Hanja");
        yield return Card(
            final, null,
            Examples.Line("먹으면", "final.off", "final.on"),
            Toggle(Settings.Bool("conversion.coda_always"), on => Change(final, Eat, Settings.Patch("conversion.coda_always", on))));

        var carried = L("그 받침을 다음 字로 넘기기", "Carry that consonant into the next syllable");
        carrySwitch = Toggle(Settings.Bool("conversion.resyllabify"), on => Change(carried, Eat, Settings.Patch("conversion.resyllabify", on)));
        carry = Card(
            carried,
            L("받침을 적을 때만 작용합니다: 위 設定, 또는 있다의 在ㅆ다.", "Only works when the consonant is written: the setting above, or 在ㅆ다 for 있다."),
            Examples.Line("먹으면", "carry.off", "carry.on"),
            carrySwitch);
        yield return carry;
    }

    protected override void Refresh()
    {
        if (carry is null || carrySwitch is null) return;
        var works = Settings.Bool("conversion.coda_always") || Settings.Str("conversion.itda") == "在ㅆ";
        carrySwitch.IsEnabled = works;
    }
}

internal sealed class TypingPage : Page
{
    public override string Id => "typing";
    public override string Title => L("入力", "Typing");
    public override string Glyph => "\uE70F";

    static readonly Sample Sky = Product.HasKun ? new("하늘을", "gksmfdmf") : new("공부를", "rhdqnfmf");

    protected override void Build()
    {
        Preview = new PreviewCard();
        Preview.Show(L("미리 보기", "Preview"), Sky);

        var start = L("앱에서 처음 打기 始作할 때", "When you start typing in an app");
        var hangulOnly = Settings.Str("conversion.script") == "hangul_only";
        // With nothing to convert to, the first choice would mean the same as the second.
        var modes = hangulOnly ? new[] { "hangul", "latin" } : new[] { "seokdok", "hangul", "latin" };
        var labels = hangulOnly
            ? new[] { L("韓㐎", "Hangul"), L("英字", "English letters") }
            : new[] { L("韓㐎, 變換함", "Hangul, converted"), L("韓㐎, 變換 無이", "Hangul, no conversion"), L("英字", "English letters") };
        var mode = Settings.Str("input.default_mode");
        var at = Array.IndexOf(modes, hangulOnly && mode == "seokdok" ? "hangul" : mode);
        Add(Card(
            start,
            L("키로 언제든 바꿀 수 있습니다. 變換할 때 쓰는 文字는 「變換」 쪽에서 定합니다.", "Keys switch this at any time. What conversion writes is set under Conversion."),
            Product.HasKun
                ? "gksmf → " + Examples.Text("start.seokdok") + " · " + Examples.Text("start.hangul") + " · " + Examples.Text("start.latin")
                : "rhdqn → " + Examples.Text("start.sino") + " · " + Examples.Text("start.sino.hangul") + " · " + Examples.Text("start.sino.latin"),
            Choice(labels, at, i => Change(start, Product.HasKun ? new Sample("gksmf", "gksmf", FollowStartMode: true) : new Sample("rhdqn", "rhdqn", FollowStartMode: true), Settings.Patch("input.default_mode", modes[i]))),
            "\uE768"));

        var space = L("Space로 變換", "Space converts");
        Add(Card(
            space,
            L("끄면 Space는 띄어쓰기이고, 「키」의 變換 키와 Tab으로 變換합니다.", "Off: Space types a space, and the convert keys under Keys or Tab convert."),
            Product.HasKun
                ? "하늘을 + Space → " + Examples.Text("space.on") + " · " + Examples.Text("space.off")
                : "공부를 + Space → " + Examples.Text("space.sino.on") + " · " + Examples.Text("space.sino.off"),
            Toggle(Settings.Bool("input.space_converts"), on => Change(space, Sky with { Trigger = Native.SpaceKey }, Settings.Patch("input.space_converts", on))),
            "\uE75D"));

        var learn = L("자주 고른 候補 먼저", "Put candidates you keep picking first");
        Add(Card(
            learn,
            L("두 번 고른 候補가 그 읽기의 첫 候補가 됩니다. 고른 낱말은 이 PC의 learned.tsv에만 남습니다.", "A candidate picked twice comes first for that reading. The words stay on this PC in learned.tsv."),
            null,
            Toggle(Settings.Bool("input.learn"), on => Change(learn, null, Settings.Patch("input.learn", on))),
            "\uE81C"));
        var forget = new Button { Content = L("지우기", "Forget") };
        forget.Click += (_, _) => forget.Content = Native.seokdok_learned_forget() ? L("지웠습니다", "Forgotten") : L("지우지 못했습니다", "Could not delete");
        Add(Card(L("고른 것 잊기", "Forget what I picked"), L("learned.tsv를 지웁니다. 끈 뒤에도 파일은 지울 때까지 남습니다.", "Deletes learned.tsv. Turning learning off keeps the file until you delete it."), null, forget, "\uE74D"));

        string[] layouts = { "vertical", "horizontal" };
        var layout = L("候補 窓의 方向", "Candidate list direction");
        Add(Card(
            layout,
            L("橫이면 ← →로 候補를 옮깁니다.", "When horizontal, ← → move through the candidates."),
            null,
            Choice(new[] { L("縱", "Vertical"), L("橫", "Horizontal") }, Array.IndexOf(layouts, Settings.Str("input.candidate_layout")),
                i => Change(layout, Sky, Settings.Patch("input.candidate_layout", layouts[i]))),
            "\uE8FD"));

        string[] styles = { "windows11", "classic" };
        var style = L("候補 窓의 모양", "Candidate list look");
        Add(Card(
            style,
            L("Windows 11은 모서리가 둥글고, 예전 Windows는 Windows 10까지처럼 네모납니다.", "Windows 11 has rounded corners. Classic is square, like Windows 10 and earlier."),
            null,
            Choice(new[] { "Windows 11", L("예전 Windows", "Classic Windows") }, Array.IndexOf(styles, Settings.Str("windows.candidate_style")),
                i => Change(style, null, Settings.Patch("windows.candidate_style", styles[i]))),
            "\uE771"));

        string[] themes = { "system", "light", "dark" };
        var theme = L("候補 窓의 色", "Candidate list colours");
        Add(Card(
            theme,
            L("高對比 테마를 쓰면 그 테마의 色으로 나옵니다.", "A high-contrast theme keeps its own colours."),
            null,
            Choice(new[] { L("시스템 設定대로", "Follow system"), L("늘 밝게", "Always light"), L("늘 어둡게", "Always dark") }, Array.IndexOf(themes, Settings.Str("windows.candidate_theme")),
                i => Change(theme, null, Settings.Patch("windows.candidate_theme", themes[i]))),
            "\uE790"));

        Add(More(
            L("더 보기", "More"),
            L("옛 韓㐎 겹字母, 앱마다 다른 글자 表示", "Old Hangul clusters, and where each app shows typing"),
            Rare(), Open.Contains(Id), on => { if (on) Open.Add(Id); else Open.Remove(Id); }));
    }

    IEnumerable<UIElement> Rare()
    {
        var clusters = L("現代 字母를 이어 옛 겹字母로", "Join modern letters into old clusters");
        yield return Card(
            clusters,
            L("ㆍ ㅿ ㆁ ㆆ는 이 設定과 關係無이 Shift로 打ㅂ니다.", "ㆍ ㅿ ㆁ ㆆ are on Shift either way."),
            "ㅅ ㄱ ㅏ → " + Examples.Text("clusters.off") + " · " + Examples.Text("clusters.on"),
            Toggle(Settings.Bool("input.old_clusters"), on => Change(clusters, new Sample("ㅅ ㄱ ㅏ", Examples.Keys("clusters.on")), Settings.Patch("input.old_clusters", on))));

        yield return Composition();
    }

    /// <summary>The `composition` table: where one application shows the text being typed.</summary>
    Border Composition()
    {
        var rows = new StackPanel { Spacing = 6 };
        foreach (var (app, surface) in Settings.Map("composition"))
        {
            var row = new Grid { ColumnSpacing = 8 };
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            var name = Text(app);
            name.VerticalAlignment = VerticalAlignment.Center;
            row.Children.Add(name);
            string[] surfaces = { "inline", "floating" };
            var choice = Choice(new[] { L("글 안에", "In the text"), L("커서 곁 窓에", "In a popup at the caret") }, Array.IndexOf(surfaces, surface),
                i => Change("", null, Settings.PatchEntry("composition", app, surfaces[i])));
            Grid.SetColumn(choice, 1);
            row.Children.Add(choice);
            var remove = new Button { Content = new FontIcon { Glyph = "\uE74D", FontSize = 14 } };
            ToolTipService.SetToolTip(remove, L("削除", "Remove"));
            remove.Click += (_, _) =>
            {
                Change("", null, Settings.PatchEntry("composition", app, null), rebuild: true);
            };
            Grid.SetColumn(remove, 2);
            row.Children.Add(remove);
            rows.Children.Add(row);
        }
        var input = new TextBox { PlaceholderText = "example.exe", MinWidth = 180 };
        var add = new Button { Content = L("앱 追加", "Add app") };
        add.Click += (_, _) =>
        {
            // The table is by executable name: a pasted path comes down to that.
            var app = System.IO.Path.GetFileName(input.Text.Trim().Trim('"'));
            if (app.Length == 0) return;
            Change("", null, Settings.PatchEntry("composition", app, "floating"), rebuild: true);
        };
        var adding = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
        adding.Children.Add(input);
        adding.Children.Add(add);
        rows.Children.Add(adding);

        var words = new StackPanel { Spacing = 8 };
        words.Children.Add(Text(L("앱마다 打는 글자를 보일 곳", "Where an app shows what you are typing")));
        words.Children.Add(Caption(L("글자가 깨지거나 밀리는 앱만 넣으세요. 넣지 않은 앱은 自動으로 定합니다.", "Add only an app that garbles typing. Others are decided automatically.")));
        words.Children.Add(rows);
        return new Border { Style = Theme("SettingCard"), Child = words };
    }
}

internal sealed class KeysPage : Page
{
    public override string Id => "keys";
    public override string Title => L("키", "Keys");
    public override string Glyph => "\uE765";

    sealed record KeyList(string Path, string Label, string Effect);

    static readonly KeyList[] Common =
    {
        new("input.keys.toggle", L("韓㐎과 英字 사이 바꾸기", "Switch between Hangul and English letters"), L("하늘 ↔ gksmf", "하늘 ↔ gksmf")),
        new("input.keys.convert", L("變換", "Convert"), Product.HasKun ? L("하늘 → 天", "하늘 → 天") : L("공부 → 工夫", "공부 → 工夫")),
        new("input.keys.script_cycle", L("韓㐎을 무엇으로 變換하나, 차례로 바꾸기", "Step through what your Hangul becomes"), Product.HasKun
            ? L("釋讀 → 國漢文混用 → 韓㐎 專用 → 釋讀. 바꾼 것은 設定에 남습니다.", "釋讀 → 國漢文混用 → Hangul only → 釋讀. The choice stays in the settings.")
            : L("國漢文混用 ↔ 韓㐎 專用. 바꾼 것은 設定에 남습니다.", "國漢文混用 ↔ Hangul only. The choice stays in the settings.")),
    };

    static readonly KeyList[] Rare =
    {
        new("input.keys.latin", L("英字로만", "To English letters only"), L("韓㐎에서 英字로 가고, 英字에서는 아무 일도 없습니다.", "Goes to English letters and does nothing if already there.")),
        new("input.keys.korean", L("韓㐎으로만", "To Hangul only"), L("英字에서 돌아옵니다. 마지막에 쓰던 쪽으로 갑니다.", "Comes back from English letters, to whichever way you last typed.")),
        new("input.keys.seokdok", L("이 앱에서 變換 켜기", "Turn conversion on in this app"), L("韓㐎을 Space로 變換하는 狀態로 갑니다.", "Back to Hangul that Space converts.")),
        new("input.keys.hangul", L("이 앱에서 變換 끄기", "Turn conversion off in this app"), L("韓㐎만 打는 狀態로 갑니다. 設定은 그대로입니다.", "Plain Hangul for now. The settings stay as they are.")),
    };

    protected override void Build()
    {
        foreach (var k in Common) Add(Editor(k));
        Add(More(
            L("더 보기", "More"),
            L("한쪽으로만 가는 키", "Keys that go one way only"),
            Rare.Select(k => (UIElement)Editor(k)).ToList(), Open.Contains(Id), on => { if (on) Open.Add(Id); else Open.Remove(Id); }));

        var reset = new Button { Content = L("키를 처음대로", "Restore the default keys") };
        reset.Click += (_, _) =>
        {
            var all = Common.Concat(Rare).Select(k => (k.Path, (JsonNode?)Settings.Array(Settings.DefaultList(k.Path)))).ToArray();
            Change("", null, Settings.Patch(all), rebuild: true);
        };
        reset.Margin = new Thickness(0, 12, 0, 0);
        Add(reset);
    }

    Border Editor(KeyList list)
    {
        var keys = Settings.List(list.Path);
        var chips = new WrapPanel();
        void Save(List<string> next)
        {
            Change("", null, Settings.Patch(list.Path, Settings.Array(next)), rebuild: true);
        }
        foreach (var key in keys)
        {
            var chip = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 2 };
            var name = Text(Keyboard.Readable(key));
            name.VerticalAlignment = VerticalAlignment.Center;
            name.Margin = new Thickness(8, 0, 0, 0);
            chip.Children.Add(name);
            if (!Native.ChordValid(key)) ToolTipService.SetToolTip(chip, L("읽지 못하는 키입니다.", "Not a key the input method understands."));
            var remove = new Button
            {
                Content = new FontIcon { Glyph = "\uE711", FontSize = 10 },
                Padding = new Thickness(6),
                Background = null,
                BorderThickness = new Thickness(0),
            };
            ToolTipService.SetToolTip(remove, L("削除", "Remove") + ": " + Keyboard.Readable(key));
            remove.Click += (_, _) =>
            {
                keys.Remove(key);
                Save(keys);
            };
            chip.Children.Add(remove);
            chips.Children.Add(new Border { Style = Theme("KeyChip"), Child = chip });
        }
        var add = new Button { Content = L("키 追加", "Add key") };
        add.Flyout = Keyboard.Picker(k =>
        {
            if (keys.Contains(k)) return;
            keys.Add(k);
            Save(keys);
        });
        chips.Children.Add(add);

        var words = new StackPanel { Spacing = 8 };
        words.Children.Add(Text(list.Label));
        words.Children.Add(Caption(list.Effect));
        words.Children.Add(chips);
        return new Border { Style = Theme("SettingCard"), Child = words };
    }
}

/// <summary>The names of `input.keys` entries as people read them.</summary>
internal static class Keyboard
{
    static readonly (string Name, string Label)[] Names =
    {
        ("Hangul", "한/영"), ("Hanja", "한자"), ("Zenkaku", "半角/全角"), ("Henkan", "変換"), ("Muhenkan", "無変換"),
        ("Kana", "カタカナ/ひらがな"), ("ImeOn", "IME On"), ("ImeOff", "IME Off"),
        ("RAlt", L("右側 Alt", "Right Alt")), ("LAlt", L("左側 Alt", "Left Alt")),
        ("RCtrl", L("右側 Ctrl", "Right Ctrl")), ("LCtrl", L("左側 Ctrl", "Left Ctrl")),
        ("RShift", L("右側 Shift", "Right Shift")), ("LShift", L("左側 Shift", "Left Shift")),
        ("Space", "Space"), ("Enter", "Enter"), ("Tab", "Tab"), ("Esc", "Esc"),
    };

    public static string Readable(string entry) =>
        string.Join(" + ", entry.Split('+').Select(part =>
        {
            var p = part.Trim();
            foreach (var (name, label) in Names)
                if (name.Equals(p, StringComparison.OrdinalIgnoreCase)) return label;
            return p;
        }));

    /// <summary>A flyout for one more key: modifiers held, and the key.</summary>
    public static Flyout Picker(Action<string> chosen)
    {
        var flyout = new Flyout();
        var panel = new StackPanel { Spacing = 10, MinWidth = 260 };
        var mods = new[] { "Ctrl", "Shift", "Alt", "Win" }.Select(m => new ToggleButton { Content = m, Tag = m }).ToList();
        var modRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 4 };
        foreach (var m in mods) modRow.Children.Add(m);
        var box = new ComboBox { HorizontalAlignment = HorizontalAlignment.Stretch, MaxDropDownHeight = 320 };
        var values = new List<string>();
        void Item(string value, string label)
        {
            values.Add(value);
            box.Items.Add(label);
        }
        foreach (var (name, label) in Names) Item(name, label);
        Item("LCtrl+LAlt", Readable("LCtrl+LAlt"));
        for (var f = 1; f <= 24; f++) Item("F" + f, "F" + f);
        for (var c = 'A'; c <= 'Z'; c++) Item(c.ToString(), c.ToString());
        for (var d = '0'; d <= '9'; d++) Item(d.ToString(), d.ToString());
        box.SelectedIndex = 0;
        var ok = new Button { Content = L("追加", "Add"), Style = Theme("AccentButtonStyle"), HorizontalAlignment = HorizontalAlignment.Right };
        var invalid = Caption(L("이 조합은 쓸 수 없습니다.", "This combination cannot be used."));
        invalid.Visibility = Visibility.Collapsed;
        string Entry() => string.Join("+", mods.Where(m => m.IsChecked == true).Select(m => (string)m.Tag).Append(values[box.SelectedIndex]));
        ok.Click += (_, _) =>
        {
            var entry = Entry();
            if (!Native.ChordValid(entry))
            {
                invalid.Visibility = Visibility.Visible;
                return;
            }
            flyout.Hide();
            chosen(entry);
        };
        panel.Children.Add(Caption(L("함께 누를 키", "Held with it")));
        panel.Children.Add(modRow);
        panel.Children.Add(Caption(L("키", "Key")));
        panel.Children.Add(box);
        panel.Children.Add(invalid);
        panel.Children.Add(ok);
        flyout.Content = panel;
        return flyout;
    }
}

internal sealed class MenuPage : Page
{
    public override string Id => "menu";
    public override string Title => L("作業 표시줄 메뉴", "Taskbar menu");
    public override string Glyph => "\uE700";

    /// <summary>What the toggles say, ahead of writes on their way to the file.</summary>
    List<string> chosen = new();

    protected override void Build()
    {
        chosen = Settings.List("windows.menu_items");
        var icon = Product.HasKun ? "釋" : "混";
        Add(Caption(L($"作業 표시줄의 {icon} 아이콘을 右클릭하면 나오는 메뉴에 보일 것을 고릅니다. 값은 메뉴에서 바로 바꿉니다.", $"Choose what appears when you right-click the {icon} icon on the taskbar. Each entry changes its setting right there.")));
        Add(Section(L("자주 쓰는 것", "Common")));
        foreach (var (key, label, where) in Items.Take(4)) Add(Item(key, label, where));
        Add(Section(L("表記", "Spelling")));
        foreach (var (key, label, where) in Items.Skip(4).Where(i => !Product.Hides(i.Key))) Add(Item(key, label, where));
    }

    static readonly (string Key, string Label, string Where)[] Items =
    {
        ("script", L("韓㐎을 무엇으로 變換하나", "What your Hangul becomes"), L("變換", "Conversion")),
        ("input_mode", L("지금 打는 方式 (韓㐎 變換, 韓㐎, 英字)", "How you are typing now (converted, Hangul, English letters)"), L("키", "Keys")),
        ("candidate_layout", L("候補 窓의 方向", "Candidate list direction"), L("入力", "Typing")),
        ("space_converts", L("Space로 變換", "Space converts"), L("入力", "Typing")),
        ("old_ha", L("하를 옛 表記로", "하 in the old spelling"), L("變換 › 하를 적는 法", "Conversion › How 하 is written")),
        ("hada_kun", L("하다·되다를 漢字로", "하다 and 되다 in Hanja"), L("變換 › 하를 적는 法", "Conversion › How 하 is written")),
        ("style", L("語尾 그대로", "The whole ending"), L("變換 › 漢字 뒤의 語尾", "Conversion › The ending after a Hanja verb")),
        ("fragment", L("남는 母音도 字母로", "With the vowel as a letter"), L("變換 › 漢字 뒤의 語尾", "Conversion › The ending after a Hanja verb")),
        ("coda_always", L("動詞의 받침을 漢字 뒤에 적기", "Write the verb's final consonant after the Hanja"), L("變換", "Conversion")),
        ("resyllabify", L("그 받침을 다음 字로 넘기기", "Carry that consonant into the next syllable"), L("變換", "Conversion")),
        ("old_clusters", L("現代 字母를 이어 옛 겹字母로", "Join modern letters into old clusters"), L("入力", "Typing")),
    };

    Border Item(string key, string label, string where)
    {
        return Card(label, L("設定 자리: ", "Setting: ") + where, null, Toggle(chosen.Contains(key), on =>
        {
            if (on) chosen.Add(key);
            else chosen.Remove(key);
            var next = Items.Select(i => i.Key).Where(k => chosen.Contains(k) && !Product.Hides(k));
            Change("", null, Settings.Patch("windows.menu_items", Settings.Array(next)));
        }));
    }
}

internal sealed class AboutPage : Page
{
    public override string Id => "about";
    public override string Title => L("情報", "About");
    public override string Glyph => "\uE946";

    protected override void Build()
    {
        var version = typeof(AboutPage).Assembly.GetName().Version;
        Add(Card(L(Product.Hanja, $"{Product.Latin} ({Product.Hanja})"), L("版 ", "Version ") + version?.ToString(3), null, null));
        Add(Card(L("説明과 使用法", "Guide"), "seokdok.mkpo.li", null, Link(Product.Guide)));
        Add(Card(L("소스와 辭典 出處", "Source and dictionary credits"), "github.com/mkpoli/seokdok", null, Link("https://github.com/mkpoli/seokdok")));
        var folder = new Button { Content = L("폴더 열기", "Open folder") };
        folder.Click += (_, _) => Program.OpenFolder();
        Add(Card(L("設定 文書", "Settings file"), L("config.toml과 custom.json이 있는 곳. 고치면 바로 反映됩니다.", "Where config.toml and custom.json live. Edits apply right away.") + "\n" + Native.ConfigDir(), null, folder));
        Add(Card(L("許可", "License"), L("코드는 MIT. 辭典마다 許可가 다릅니다.", "The code is MIT. Each dictionary has its own license."), null, null));
    }

    static HyperlinkButton Link(string url) => new() { Content = url.Replace("https://", "").TrimEnd('/'), NavigateUri = new Uri(url) };
}
