using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json.Nodes;

namespace SeokdokSettings;

/// <summary>products/&lt;id&gt;.json, built into the app: its names, and what the settings offer.</summary>
internal static class Product
{
    static readonly JsonNode Root = Load();

    static JsonNode Load()
    {
        using var stream = typeof(Product).Assembly.GetManifestResourceStream("product.json")!;
        using var reader = new StreamReader(stream);
        return JsonNode.Parse(reader.ReadToEnd())!;
    }

    public static string Id => Root["id"]!.GetValue<string>();
    public static string Hanja => Root["hanja"]!.GetValue<string>();
    public static string Latin => Root["latin"]!.GetValue<string>();
    public static string Guide => Root["guide"]!.GetValue<string>();

    public static readonly IReadOnlyList<string> Scripts = Root["scripts"]!.AsArray().Select(n => n!.GetValue<string>()).ToList();
    static readonly HashSet<string> Hidden = Root["hidden_settings"]!.AsArray().Select(n => n!.GetValue<string>()).ToHashSet();

    public static bool Hides(string setting) => Hidden.Contains(setting);

    /// <summary>Whether it writes 訓讀 at all. Without it, examples show what it does write.</summary>
    public static bool HasKun => Scripts.Contains("seokdok");
}
