using System;
using System.IO;
using Microsoft.UI.Xaml;

namespace SeokdokSettings;

public partial class App : Application
{
    public App()
    {
        InitializeComponent();
        UnhandledException += (_, e) => Log(e.Message + "\n" + e.Exception);
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args) => Program.Open();

    /// <summary>What went wrong, for a bug report: the product's settings log in the temporary folder.</summary>
    public static void Log(string message)
    {
        try { File.AppendAllText(Path.Combine(Path.GetTempPath(), Product.Id + "-settings.log"), $"{DateTime.Now:O} {message}\n"); }
        catch (IOException) { }
    }
}
