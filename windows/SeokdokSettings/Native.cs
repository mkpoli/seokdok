using System;
using System.Runtime.InteropServices;

namespace SeokdokSettings;

/// <summary>The C interface of crates/seokdok-ffi, loaded from seokdok_ffi.dll beside the app.</summary>
internal static partial class Native
{
    const string Dll = "seokdok_ffi.dll";

    public const int Letter = 1, SpaceKey = 3, Enter = 4, Convert = 14;

    [LibraryImport(Dll, StringMarshalling = StringMarshalling.Utf8)] public static partial IntPtr seokdok_new(string? dictDir);
    [LibraryImport(Dll)] public static partial void seokdok_sync(IntPtr ime);
    [LibraryImport(Dll)][return: MarshalAs(UnmanagedType.U1)] public static partial bool seokdok_key(IntPtr ime, int kind, uint ch, [MarshalAs(UnmanagedType.U1)] bool shift);
    [LibraryImport(Dll)] public static partial void seokdok_reset(IntPtr ime);
    [LibraryImport(Dll)] public static partial IntPtr seokdok_take_commit(IntPtr ime);
    [LibraryImport(Dll)] public static partial nuint seokdok_popup_len(IntPtr ime);
    [LibraryImport(Dll)] public static partial IntPtr seokdok_popup_item(IntPtr ime, nuint i, IntPtr note);
    [LibraryImport(Dll)] public static partial void seokdok_popup_info(IntPtr ime, out nuint selected, out nuint position, out nuint total, [MarshalAs(UnmanagedType.U1)] out bool horizontal);
    [LibraryImport(Dll)] public static partial void seokdok_set_mode(IntPtr ime, int mode);
    [LibraryImport(Dll)] public static partial void seokdok_reload(IntPtr ime);
    [LibraryImport(Dll)] public static partial void seokdok_set_private_field(IntPtr ime, [MarshalAs(UnmanagedType.U1)] bool privateField);
    [LibraryImport(Dll)][return: MarshalAs(UnmanagedType.U1)] public static partial bool seokdok_learned_forget();
    [LibraryImport(Dll)][return: MarshalAs(UnmanagedType.U1)] public static partial bool seokdok_ready();

    [LibraryImport(Dll)] private static partial IntPtr seokdok_config_json();
    [LibraryImport(Dll)] private static partial IntPtr seokdok_config_defaults_json();
    [LibraryImport(Dll)] private static partial IntPtr seokdok_config_error();
    [LibraryImport(Dll)] private static partial IntPtr seokdok_config_dir();
    [LibraryImport(Dll, StringMarshalling = StringMarshalling.Utf8)] private static partial IntPtr seokdok_config_update_json(string patch);
    [LibraryImport(Dll, StringMarshalling = StringMarshalling.Utf8)][return: MarshalAs(UnmanagedType.U1)] private static partial bool seokdok_chord_valid(string text);
    [LibraryImport(Dll)] private static partial IntPtr seokdok_custom_json();
    [LibraryImport(Dll, StringMarshalling = StringMarshalling.Utf8)] private static partial IntPtr seokdok_custom_save_json(string json);
    [LibraryImport(Dll)] private static partial void seokdok_string_free(IntPtr s);

    /// <summary>A string the session allocated, freed here; null for NULL.</summary>
    static string? Owned(IntPtr p)
    {
        if (p == IntPtr.Zero) return null;
        var text = Marshal.PtrToStringUTF8(p);
        seokdok_string_free(p);
        return text;
    }

    /// <summary>A string the session keeps; valid until the view changes.</summary>
    public static string Borrowed(IntPtr p) => p == IntPtr.Zero ? "" : Marshal.PtrToStringUTF8(p) ?? "";

    public static string ConfigJson() => Owned(seokdok_config_json()) ?? "{}";
    public static string DefaultsJson() => Owned(seokdok_config_defaults_json()) ?? "{}";
    public static string? ConfigError() => Owned(seokdok_config_error());
    public static string? ConfigDir() => Owned(seokdok_config_dir());
    public static string? UpdateConfig(string patch) => Owned(seokdok_config_update_json(patch));
    public static bool ChordValid(string text) => seokdok_chord_valid(text);
    public static string? CustomJson() => Owned(seokdok_custom_json());
    public static string? SaveCustom(string json) => Owned(seokdok_custom_save_json(json));
}
