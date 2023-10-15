namespace JukeBoxDesktop;

static class Program
{
    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    static void Main()
    {
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        Console.WriteLine("Test");
        // var mainForm = new MainForm();
        Console.WriteLine("Test2");
        Application.Run();
    }
}