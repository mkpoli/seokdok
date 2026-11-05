using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Nodes;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Storage.Pickers;
using static SeokdokSettings.Ui;

namespace SeokdokSettings;

/// <summary>
/// custom.json: own phrases, own words that inflect, and rewrite rules, the
/// same document as the site's Customize section. Fields this page does not
/// show stay in the file.
/// </summary>
internal sealed class WordsPage : Page
{
    public override string Id => "words";
    public override string Title => L("自作 個言", "My words");
    public override string Glyph => "\uE8AC";

    public override bool FollowsFile => false;

    static string tab = "phrases";
    JsonObject? doc;
    bool dirty;

    /// <summary>The file as a document, or null when it holds something this page must not overwrite.</summary>
    static JsonObject? Load()
    {
        var text = Native.CustomJson();
        var dir = Native.ConfigDir();
        // There, but not text.
        if (text is null && dir is not null && File.Exists(Path.Combine(dir, "custom.json"))) return null;
        JsonObject? read;
        try { read = JsonNode.Parse(text ?? "{}") as JsonObject; }
        catch (JsonException) { return null; }
        if (read is null) return null;
        read["format"] = "seokdok-custom";
        read["version"] ??= 1;
        foreach (var list in new[] { "phrases", "words", "rules" })
        {
            read[list] ??= new JsonArray();
            if (read[list] is not JsonArray entries || entries.Any(e => e is not JsonObject)) return null;
        }
        return read;
    }

    /// <summary>The document stays as edited across rebuilds of the page, and is read when the page opens.</summary>
    public override void Enter() => doc = Load();

    public override void Leave()
    {
        if (!dirty) return;
        dirty = false;
        Save();
    }

    protected override void Build()
    {
        if (doc is null)
        {
            Add(Caption(L("custom.json을 讀지 못해 고칠 수 없습니다. 文書를 고친 뒤 이 쪽을 다시 여세요.", "custom.json cannot be read, so it cannot be edited here. Fix the file, then open this page again.")));
            var folder = new Button { Content = L("폴더 열기", "Open folder") };
            folder.Click += (_, _) => Program.OpenFolder();
            Add(folder);
            return;
        }
        var bar = new SelectorBar();
        foreach (var (id, label) in new[] { ("phrases", L("自作 個言", "Phrases")), ("words", L("訓讀 個言", "Words that inflect")), ("rules", L("規則", "Rules")), ("share", L("共有", "Share")) }.Where(t => t.Item1 != "words" || !Product.Hides("kun_words")))
        {
            var item = new SelectorBarItem { Text = label, Tag = id };
            bar.Items.Add(item);
            if (id == tab) bar.SelectedItem = item;
        }
        bar.SelectionChanged += (_, _) =>
        {
            tab = (string)bar.SelectedItem.Tag;
            Leave();
            Rebuild?.Invoke();
        };
        Add(bar);
        switch (tab)
        {
            case "words": Words(); break;
            case "rules": Rules(); break;
            case "share": Share(); break;
            default: Phrases(); break;
        }
    }

    void Save()
    {
        dirty = false;
        var problem = Native.SaveCustom(Text());
        if (problem is not null) Report?.Invoke(problem);
        else Engine.Shared.ReloadCustom();
    }

    string Text() => doc!.ToJsonString(new JsonSerializerOptions { WriteIndented = true, Encoder = System.Text.Encodings.Web.JavaScriptEncoder.UnsafeRelaxedJsonEscaping });

    JsonArray List(string name) => (JsonArray)doc![name]!;

    /// <summary>A text field of an entry; anything else in a hand-written file reads as empty.</summary>
    static string Str(JsonObject entry, string key) =>
        entry[key] is JsonValue v && v.TryGetValue<string>(out var s) ? s : "";

    static bool Flag(JsonObject entry, string key, bool otherwise) =>
        entry[key] is JsonValue v && v.TryGetValue<bool>(out var b) ? b : otherwise;

    /// <summary>A text field bound to a field of an entry, saved when focus leaves it or the page goes.</summary>
    TextBox Field(JsonObject entry, string key, string header, double width = 200)
    {
        var box = new TextBox { Header = header, Text = Str(entry, key), MinWidth = width };
        box.TextChanged += (_, _) =>
        {
            if (Str(entry, key) == box.Text) return;
            entry[key] = box.Text;
            dirty = true;
        };
        box.LostFocus += (_, _) => Leave();
        return box;
    }

    /// <summary>One entry: its fields, whether it is on, and a remove button.</summary>
    Border Entry(string list, JsonObject entry, params FrameworkElement[] fields)
    {
        var row = new WrapPanel();
        foreach (var f in fields) row.Children.Add(f);
        var on = Toggle(Flag(entry, "on", true), v =>
        {
            entry["on"] = v;
            Save();
        });
        var remove = new Button { Content = new FontIcon { Glyph = "\uE74D", FontSize = 14 }, VerticalAlignment = VerticalAlignment.Center };
        ToolTipService.SetToolTip(remove, L("削除", "Remove"));
        remove.Click += (_, _) =>
        {
            List(list).Remove(entry);
            Save();
            Rebuild?.Invoke();
        };
        var grid = new Grid { ColumnSpacing = 12 };
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        grid.Children.Add(row);
        Grid.SetColumn(on, 1);
        grid.Children.Add(on);
        Grid.SetColumn(remove, 2);
        grid.Children.Add(remove);
        return new Border { Style = Theme("SettingCard"), Child = grid };
    }

    void AddButton(string label, string list, Func<JsonObject> fresh, string empty)
    {
        if (List(list).Count == 0) Add(Caption(empty));
        var add = new Button { Content = label, Margin = new Thickness(0, 8, 0, 0) };
        add.Click += (_, _) =>
        {
            List(list).Add((JsonNode)fresh());
            Rebuild?.Invoke();
        };
        Add(add);
    }

    static IEnumerable<JsonObject> Entries(JsonArray list) => list.OfType<JsonObject>().ToList();

    void Phrases()
    {
        Add(Caption(L("讀는 音에 定해진 字를 붙입니다. 候補의 첫째로 나옵니다. 풀어읽기모임 → 釋讀會", "Fixed text for a reading. It comes first among the candidates. 풀어읽기모임 → 釋讀會")));
        foreach (var p in Entries(List("phrases")))
            Add(Entry("phrases", p, Field(p, "reading", L("讀音 (韓㐎)", "Reading (Hangul)")), Field(p, "text", L("表記", "Written"))));
        AddButton(L("個言 追加", "Add phrase"), "phrases", () => new JsonObject { ["on"] = true, ["reading"] = "", ["text"] = "" }, L("自作 個言이 없습니다.", "No phrases yet."));
    }

    void Words()
    {
        Add(Caption(L("活用하는 個言입니다. 짓다 → 作다를 動詞로 넣으면 지었다는 作었다로 變換됩니다.", "Words that inflect. With 짓다 → 作다 as a verb, 지었다 converts to 作었다.")));
        string[] parts = { "noun", "pron", "adv", "det", "verb", "adj" };
        string[] partNames = { L("名詞", "noun"), L("代名詞", "pronoun"), L("副詞", "adverb"), L("冠形詞", "determiner"), L("動詞", "verb"), L("形容詞", "adjective") };
        foreach (var w in Entries(List("words")))
        {
            var part = Choice(partNames, Math.Max(0, Array.IndexOf(parts, Str(w, "pos"))), i =>
            {
                w["pos"] = parts[i];
                Save();
            });
            part.Header = L("品詞", "Part of speech");
            part.MinWidth = 120;
            Add(Entry("words", w, Field(w, "lemma", L("基本形", "Dictionary form")), Field(w, "written", L("釋讀 表記", "Written in Hanja")), part));
        }
        AddButton(L("個言 追加", "Add word"), "words", () => new JsonObject { ["on"] = true, ["lemma"] = "", ["written"] = "", ["pos"] = "verb" }, L("訓讀 個言이 없습니다.", "No words yet."));
    }

    void Rules()
    {
        Add(Caption(L("글자를 바꾸는 規則입니다. 위에서부터 차례로 適用합니다. 入力 規則은 變換 前의 韓㐎을, 出力 規則은 候補마다 바꿉니다.", "Rules that rewrite text, run top to bottom. Input rules rewrite the Hangul before conversion; output rules rewrite every candidate.")));
        foreach (var r in Entries(List("rules")))
        {
            var stage = Choice(new[] { L("入力 (變換 前)", "Input (before converting)"), L("出力 (候補)", "Output (candidates)") }, Str(r, "stage") == "input" ? 0 : 1, i =>
            {
                r["stage"] = i == 0 ? "input" : "output";
                Save();
            });
            stage.Header = L("段階", "Stage");
            var mode = Choice(new[] { L("바꾸기", "Replace"), L("더하기 (原來 候補도 남김)", "Add (keeps the original)") }, Str(r, "mode") == "derive" ? 1 : 0, i =>
            {
                r["mode"] = i == 1 ? "derive" : "xform";
                Save();
            });
            mode.Header = L("方式", "Mode");
            var regex = new CheckBox { Content = L("正規式", "Regular expression"), IsChecked = Flag(r, "regex", false), VerticalAlignment = VerticalAlignment.Bottom };
            regex.Click += (_, _) =>
            {
                r["regex"] = regex.IsChecked == true;
                Save();
            };
            Add(Entry("rules", r, Field(r, "pattern", L("찾을 字", "Find"), 160), Field(r, "replace", L("바꿀 字", "Replace with"), 160), stage, mode, regex));
        }
        AddButton(L("規則 追加", "Add rule"), "rules", () => new JsonObject { ["on"] = true, ["stage"] = "output", ["mode"] = "xform", ["regex"] = false, ["pattern"] = "", ["replace"] = "" }, L("規則이 없습니다.", "No rules yet."));
    }

    void Share()
    {
        var import = new Button { Content = L("文書 열기", "Open a file") };
        import.Click += async (_, _) =>
        {
            var picker = new FileOpenPicker();
            WinRT.Interop.InitializeWithWindow.Initialize(picker, Program.WindowHandle);
            picker.FileTypeFilter.Add(".json");
            var file = await picker.PickSingleFileAsync();
            if (file is null) return;
            string? problem;
            try { problem = Native.SaveCustom(File.ReadAllText(file.Path)); }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { problem = e.Message; }
            if (problem is not null)
            {
                Report?.Invoke(problem);
                return;
            }
            Enter();
            Engine.Shared.ReloadCustom();
            Rebuild?.Invoke();
        };
        Add(Card(L("JSON 들여오기", "Import JSON"), L("들여오면 只今의 設定을 代身합니다.", "Importing replaces what is here now."), "seokdok.mkpo.li", import));
        var export = new Button { Content = L("文書로 貯藏", "Save as a file") };
        export.Click += async (_, _) =>
        {
            var picker = new FileSavePicker { SuggestedFileName = "custom" };
            WinRT.Interop.InitializeWithWindow.Initialize(picker, Program.WindowHandle);
            picker.FileTypeChoices.Add("JSON", new List<string> { ".json" });
            var file = await picker.PickSaveFileAsync();
            if (file is null) return;
            try { File.WriteAllText(file.Path, Text()); }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { Report?.Invoke(e.Message); }
        };
        Add(Card(L("JSON 내보내기", "Export JSON"), L("規則과 個言을 共有합니다.", "Share your rules and words."), null, export));
    }
}
