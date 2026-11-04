using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json.Nodes;

namespace SeokdokSettings;

/// <summary>
/// The settings as the Rust session reads them. Every read and write goes
/// through it; here is only the shape of the JSON. A change names just what it
/// changes, so whatever another application wrote meanwhile survives.
/// </summary>
internal static class Settings
{
    static JsonNode? current;

    public static string Text { get; private set; } = "{}";

    /// <summary>Why the file cannot be changed, when it cannot.</summary>
    public static string? Problem { get; private set; }

    /// <summary>Reads the file again. True when it says something new.</summary>
    public static bool Reload()
    {
        Problem = Native.ConfigError();
        var text = Native.ConfigJson();
        var changed = text != Text;
        Text = text;
        current = JsonNode.Parse(text);
        return changed;
    }

    static JsonNode? At(JsonNode? root, string path)
    {
        var node = root;
        foreach (var part in path.Split('.')) node = node?[part];
        return node;
    }

    public static bool Bool(string path) => At(current, path)?.GetValue<bool>() ?? false;
    public static string Str(string path) => At(current, path)?.GetValue<string>() ?? "";
    public static int Int(string path) => At(current, path)?.GetValue<int>() ?? 0;

    public static List<string> List(string path) =>
        (At(current, path) as JsonArray)?.Select(n => n?.GetValue<string>() ?? "").ToList() ?? new();

    public static List<string> DefaultList(string path) =>
        (At(JsonNode.Parse(Native.DefaultsJson()), path) as JsonArray)?.Select(n => n?.GetValue<string>() ?? "").ToList() ?? new();

    public static List<KeyValuePair<string, string>> Map(string path) =>
        (At(current, path) as JsonObject)?.Select(p => KeyValuePair.Create(p.Key, p.Value?.GetValue<string>() ?? "")).ToList() ?? new();

    /// <summary>A merge patch holding one value at a dotted path.</summary>
    public static string Patch(string path, JsonNode? value)
    {
        var parts = path.Split('.');
        JsonNode? node = value;
        foreach (var part in parts.Reverse()) node = new JsonObject { [part] = node };
        return node!.ToJsonString();
    }

    /// <summary>A merge patch for one key of a table; the key may hold dots, as a file name does.</summary>
    public static string PatchEntry(string table, string key, JsonNode? value) =>
        new JsonObject { [table] = new JsonObject { [key] = value } }.ToJsonString();

    /// <summary>A merge patch holding several values.</summary>
    public static string Patch(params (string Path, JsonNode? Value)[] values)
    {
        var root = new JsonObject();
        foreach (var (path, value) in values)
        {
            var parts = path.Split('.');
            var node = root;
            foreach (var part in parts[..^1])
            {
                if (node[part] is not JsonObject next) node[part] = next = new JsonObject();
                node = next;
            }
            node[parts[^1]] = value;
        }
        return root.ToJsonString();
    }

    public static JsonArray Array(IEnumerable<string> items) => new(items.Select(i => (JsonNode?)JsonValue.Create(i)).ToArray());
}
