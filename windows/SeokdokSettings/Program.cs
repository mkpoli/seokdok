using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.Windows.AppLifecycle;
using WinRT;

namespace SeokdokSettings;

/// <summary>One window per user: a second launch brings the first to the front and leaves.</summary>
internal static partial class Program
{
    public static DispatcherQueue? Queue { get; private set; }
    public static IntPtr WindowHandle { get; private set; }

    static MainWindow? window;

    [STAThread]
    static void Main()
    {
        AppDomain.CurrentDomain.UnhandledException += (_, e) => App.Log(e.ExceptionObject.ToString() ?? "");
        ComWrappersSupport.InitializeComWrappers();
        var instance = AppInstance.FindOrRegisterForKey("seokdok-settings");
        if (!instance.IsCurrent)
        {
            AllowSetForegroundWindow(instance.ProcessId);
            instance.RedirectActivationToAsync(AppInstance.GetCurrent().GetActivatedEventArgs()).AsTask().Wait();
            return;
        }
        instance.Activated += (_, _) => Queue?.TryEnqueue(() =>
        {
            window?.Activate();
            SetForegroundWindow(WindowHandle);
        });
        Application.Start(start =>
        {
            Queue = DispatcherQueue.GetForCurrentThread();
            SynchronizationContext.SetSynchronizationContext(new DispatcherQueueSynchronizationContext(Queue));
            new App();
        });
    }

    public static void Open()
    {
        window = new MainWindow();
        WindowHandle = WinRT.Interop.WindowNative.GetWindowHandle(window);
        window.Activate();
    }

    public static void OpenFolder()
    {
        var dir = Native.ConfigDir();
        if (dir is not null) Process.Start(new ProcessStartInfo(dir) { UseShellExecute = true });
    }

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool AllowSetForegroundWindow(uint process);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool SetForegroundWindow(IntPtr window);
}
