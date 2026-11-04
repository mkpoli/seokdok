using System;
using System.IO;
using System.Linq;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Graphics;

namespace SeokdokSettings;

public sealed partial class MainWindow : Window
{
    readonly Page[] pages;
    Page? page;

    public MainWindow()
    {
        InitializeComponent();
        Title = Ui.L("釋讀 設定", "Seokdok settings");
        BarTitle.Text = Title;
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(Bar);
        Root.Language = "ko";

        var icon = Path.Combine(AppContext.BaseDirectory, "icon.ico");
        if (File.Exists(icon))
        {
            AppWindow.SetIcon(icon);
            Mark.Source = new BitmapImage(new Uri(icon));
        }
        Size(1040, 760);
        if (AppWindow.Presenter is OverlappedPresenter window)
        {
            var scale = Content?.XamlRoot?.RasterizationScale ?? Dpi() / 96.0;
            window.PreferredMinimumWidth = (int)(360 * scale);
            window.PreferredMinimumHeight = (int)(420 * scale);
        }

        pages = new Page[] { new ConversionPage(), new TypingPage(), new KeysPage(), new WordsPage(), new MenuPage(), new AboutPage() };
        foreach (var p in pages)
        {
            p.Report = Show;
            p.Rebuild = Rebuild;
            Nav.MenuItems.Add(new NavigationViewItem { Content = p.Title, Tag = p.Id, Icon = new FontIcon { Glyph = p.Glyph } });
        }
        Nav.DisplayModeChanged += (_, e) =>
        {
            foreach (var p in pages) p.SetNarrow(e.DisplayMode == NavigationViewDisplayMode.Minimal);
        };
        Nav.SelectionChanged += (_, e) =>
        {
            if (e.SelectedItem is NavigationViewItem item) Go((string)item.Tag);
        };
        Closed += (_, _) =>
        {
            page?.Leave();
            Engine.Shared.Finish();
        };
        Activated += (_, e) =>
        {
            // Another application or an editor may have changed the file meanwhile.
            if (noLibrary) return;
            if (e.WindowActivationState != WindowActivationState.Deactivated && Page.Writing == 0 && Settings.Reload() && page is { FollowsFile: true }) Rebuild();
            ShowFileProblem();
        };

        try
        {
            Settings.Reload();
            Nav.SelectedItem = Nav.MenuItems[0];
            try { WatchFile(); }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { App.Log(e.ToString()); }
        }
        catch (Exception e) when (e is DllNotFoundException or EntryPointNotFoundException)
        {
            Show(Ui.L("seokdok_ffi.dll을 찾지 못했습니다. 釋讀을 다시 設置해 주세요.", "seokdok_ffi.dll was not found. Please reinstall Seokdok."));
            Nav.IsEnabled = false;
            noLibrary = true;
            return;
        }
        ShowFileProblem();
    }

    bool noLibrary;

    /// <summary>
    /// The taskbar menu and the rotation key write config.toml from other
    /// applications, or from this window's own text fields. A choice made here
    /// names only its own key, so what they wrote is never overwritten; this
    /// keeps what is on show true as well.
    /// </summary>
    void WatchFile()
    {
        var dir = Native.ConfigDir();
        if (dir is null) return;
        Directory.CreateDirectory(dir);
        // The watcher calls from the thread pool. Enqueue a lambda: a method of a WinRT
        // object as the delegate fails to marshal.
        var queue = DispatcherQueue;
        var timer = queue.CreateTimer();
        timer.Interval = TimeSpan.FromMilliseconds(300);
        timer.IsRepeating = false;
        timer.Tick += (_, _) =>
        {
            if (Page.Writing > 0)
            {
                timer.Start();
                return;
            }
            if (Settings.Reload() && page is { FollowsFile: true }) Rebuild();
            ShowFileProblem();
        };
        watcher = new FileSystemWatcher(dir, "config.toml*") { EnableRaisingEvents = true };
        void Changed() => queue.TryEnqueue(() => timer.Start());
        watcher.Changed += (_, _) => Changed();
        watcher.Created += (_, _) => Changed();
        watcher.Renamed += (_, _) => Changed();
        watcher.Deleted += (_, _) => Changed();
    }

    FileSystemWatcher? watcher;

    uint Dpi() => GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this));

    [System.Runtime.InteropServices.LibraryImport("user32.dll")]
    private static partial uint GetDpiForWindow(IntPtr window);

    void Size(int width, int height)
    {
        var scale = Dpi() / 96.0;
        var area = DisplayArea.GetFromWindowId(AppWindow.Id, DisplayAreaFallback.Primary).WorkArea;
        var w = Math.Min((int)(width * scale), area.Width);
        var h = Math.Min((int)(height * scale), area.Height);
        AppWindow.MoveAndResize(new RectInt32(area.X + (area.Width - w) / 2, area.Y + (area.Height - h) / 2, w, h));
    }

    void Go(string id)
    {
        page?.Leave();
        page = pages.First(p => p.Id == id);
        page.Enter();
        Rebuild();
    }

    void Rebuild()
    {
        if (page is not null) Nav.Content = page.Render();
    }

    void ShowFileProblem()
    {
        if (Settings.Problem is null)
        {
            if (fileProblem) Problem.IsOpen = false;
            fileProblem = false;
            Nav.IsEnabled = true;
            return;
        }
        fileProblem = true;
        Show(Ui.L("config.toml을 讀지 못해 設定을 바꿀 수 없습니다. 文書를 고친 뒤 이 窓으로 돌아오세요.", "config.toml cannot be read, so settings cannot be changed. Fix the file, then come back to this window.") + "\n" + Settings.Problem);
        Nav.IsEnabled = false;
    }

    bool fileProblem;

    void Show(string message)
    {
        Problem.Message = message;
        Problem.IsOpen = true;
    }
}
