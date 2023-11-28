using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.IO.Ports;
using System.Management;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;

using LibreHardwareMonitor.Hardware;

namespace JukeBoxDesktop
{
    public partial class MainForm : Form
    {
        // For updating all the stats
        private Computer comp = new Computer
        {
            IsCpuEnabled = true,
            IsGpuEnabled = true,
            IsMemoryEnabled = true,
        };
        private readonly UpdateVisitor _visitor = new UpdateVisitor();
        
        // For the pop up windows
        private const string PopupFailedToOpen =
            "JukeBox Desktop Error: Failed to open serial device.";
        private const string PopupNotActualSerial =
            "The serial device selected could not be opened. It" +
            " may not have been an actual serial device.\n\nSorry!";
        private const string PopupSerialInUse =
            "The serial device selected could not be opened. It" +
            " may already be in use by another program.\n\nSorry!";
        private const string PopupSerialFailed =
            "The serial device selected is not responding to JukeBox Desktop. This" +
            " could be due to the device not recognizing serial or the chip" +
            " on the board may have gone bad and no longer functions.\n\nSorry!";

        // For closing via systray
        private bool _closing = false;

        // All the stats we send via serial.
        private string cpuName = "Unknown CPU";
        private string cpuFreq = "0";
        private string cpuTemp = "0";
        private string cpuLoad = "0";

        private string gpuName      = "Unknown GPU";
        private string gpuTemp      = "0";
        private string gpuCoreLoad  = "0";
        private string gpuCoreClock = "0";
        private string gpuVramLoad  = "0";
        private string gpuVramClock = "0";

        private string ramUsed  = "0";
        private string ramTotal = "0";

        // The stages of denial-- I mean communications.
        enum SerialStage
        {
            GreetHost,
            GreetDevice,
            RecvParts,
            RecvConfirm,
            ContStats,
            ContConfrim,
        }

        // Serial comms variables.
        private bool _continueComms = true;
        private bool _sendmessage = false;
        private readonly object _compMutex = new object();
        private SerialPort serial = new SerialPort();
        private Thread comms;
        private SerialStage stage = SerialStage.GreetHost;
        // private byte stagepart = 0;

        public MainForm()
        {
            InitializeComponent();
            comp.Open();

            // Run one update to populate stat values.
            UpdateComp();

            // Set up serial comms.
            // Serial.begin(115200, SERIAL_8E2);
            comms = new Thread(SerialComms);
            serial.BaudRate = 115200;
            serial.DataBits = 8;
            serial.Parity = Parity.Even;
            serial.StopBits = StopBits.Two;
            serial.Handshake = Handshake.None;
            serial.DtrEnable = true;
            serial.RtsEnable = true;

            comms.Start();
        }

        private void UpdateComp()
        {
            // Wait turn for mutex
            lock (_compMutex)
            {
                // Update the sensors
                comp.Accept(_visitor);

                // Iterate through all the hardware
                foreach (var hardware in comp.Hardware)
                {
                    // Let's check each of the hardwarez
                    if (hardware.HardwareType == HardwareType.Cpu)
                    {
                        // First up is CPU, let's grab the average frequency, temperature, and load.

                        decimal _freqSum = 0;
                        decimal _cpuTemp = 0;
                        decimal _cpuLoad = 0;
                        byte _coreCount = 0;

                        foreach (var sensor in hardware.Sensors)
                        {
                            if (sensor.SensorType == SensorType.Clock && sensor.Name != "Bus Speed")
                            {
                                _coreCount++;
                                _freqSum += (decimal)sensor.Value;
                            }
                            else if (sensor.SensorType == SensorType.Temperature && 
                                (sensor.Name == "Core (Tctl)" || sensor.Name == "Core (Tctl/Tdie)"))
                            {
                                _cpuTemp = (decimal)sensor.Value;
                            }
                            else if (sensor.SensorType == SensorType.Load && sensor.Name == "CPU Total")
                            {
                                _cpuLoad = (decimal)sensor.Value;
                            }
                        }

                        cpuName = hardware.Name;
                        cpuFreq = (_freqSum / _coreCount / 1000).ToString("n2");
                        cpuTemp = _cpuTemp.ToString("f1");
                        cpuLoad = _cpuLoad.ToString("f1").PadLeft(4, ' ');
                    }
                    else if (hardware.HardwareType == HardwareType.GpuNvidia || hardware.HardwareType == HardwareType.GpuAmd)
                    {
                        // Now for gpu. We grab the clock speed of the core and ram, as well as their load.

                        decimal _gpuTemp = 0;

                        decimal _gpuCoreLoad = 0;
                        decimal _gpuCoreClock = 0;

                        decimal _gpuVramLoad = 0;
                        decimal _gpuVramClock = 0;

                        foreach (var sensor in hardware.Sensors)
                        {
                            if (sensor.SensorType == SensorType.Clock)
                            {
                                if (sensor.Name == "GPU Core")
                                    _gpuCoreClock = (decimal)sensor.Value;
                                else if (sensor.Name == "GPU Memory")
                                    _gpuVramClock = (decimal)sensor.Value;
                                /*else if (sensor.Name == "GPU Shader")
                                    gpuShaderClock = (decimal)sensor.Value;*/
                            }
                            else if (sensor.SensorType == SensorType.Temperature && sensor.Name == "GPU Core")
                            {
                                _gpuTemp = (decimal)sensor.Value;
                            }
                            else if (sensor.SensorType == SensorType.Load)
                            {
                                if (sensor.Name == "GPU Core")
                                    _gpuCoreLoad = (decimal)sensor.Value;
                                else if (sensor.Name == "GPU Memory")
                                    _gpuVramLoad = (decimal)sensor.Value;
                            }
                            /*else if (sensor.SensorType == SensorType.SmallData)
                            {
                                if (sensor.Name == "GPU Memory Used")
                                    gpuMemoryUsed = (decimal)sensor.Value;
                                else if (sensor.Name == "GPU Memory Total")
                                    gpuMemoryTotal = (decimal)sensor.Value;
                            }*/
                        }

                        gpuName = hardware.Name;
                        gpuTemp = _gpuTemp.ToString("f1");
                        gpuCoreLoad = _gpuCoreLoad.ToString("f1").PadLeft(4, ' ');
                        gpuCoreClock = _gpuCoreClock.ToString("f1").PadLeft(6, ' ');
                        gpuVramLoad = _gpuVramLoad.ToString("f1").PadLeft(4, ' ');
                        gpuVramClock = _gpuVramClock.ToString("f1").PadLeft(6, ' ');
                    }
                    else if (hardware.HardwareType == HardwareType.Memory)
                    {
                        decimal _ramUsed = 0;
                        decimal _ramAvailable = 0;

                        foreach (var sensor in hardware.Sensors)
                        {
                            if (sensor.SensorType == SensorType.Data)
                            {
                                if (sensor.Name == "Memory Used")
                                    _ramUsed = (decimal)sensor.Value;
                                else if (sensor.Name == "Memory Available")
                                    _ramAvailable = (decimal)sensor.Value;
                            }
                        }

                        ramUsed = _ramUsed.ToString("f1");
                        ramTotal = Decimal.Round(_ramUsed + _ramAvailable).ToString("n0");
                    }
                }

                // Check if we need to update all the values for the GUI
                if (Visible)
                {
                    labelCpuName.Text = $"Name: {cpuName}";
                    labelCpuFreq.Text = $"Freq: {cpuFreq} GHz";
                    labelCpuTemp.Text = $"Temp: {cpuTemp} ° C";
                    labelCpuLoad.Text = $"Load: {cpuLoad} %";

                    labelGpuName.Text = $"Name: {gpuName}";
                    labelGpuTemp.Text = $"Temp: {gpuTemp} ° C";
                    labelGpuCoreLoad.Text = $"Core Load: {gpuCoreLoad} %";
                    labelGpuCoreClock.Text = $"Core Clock: {gpuCoreClock} MHz";
                    labelGpuVramLoad.Text = $"VRAM Load: {gpuVramLoad} % used";
                    labelGpuVramClock.Text = $"VRAM Clock: {gpuVramClock} MHz";

                    labelRamUsed.Text = $"Used: {ramUsed} GB";
                    labelRamTotal.Text = $"Total: {ramTotal} GB";

                    // Show on screen.
                    labelCpuName.Refresh();
                    labelCpuFreq.Refresh();
                    labelCpuTemp.Refresh();
                    labelCpuLoad.Refresh();

                    labelGpuName.Refresh();
                    labelGpuTemp.Refresh();
                    labelGpuCoreLoad.Refresh();
                    labelGpuCoreClock.Refresh();
                    labelGpuVramClock.Refresh();

                    labelRamUsed.Refresh();
                    labelRamTotal.Refresh();
                }
            } // Unlock mutex!
        }

        private void SerialComms()
        {
            while (_continueComms)
            {
                // If serial is started, don't try to use it.
                // TODO: lock the serial object when doing comms so it doesnt get interrupted mid-write.
                if (!serial.IsOpen)
                {
                    Thread.Sleep(1000); // just so CPU usage doesn't spike.
                    continue;
                }

                lock (serial)
                {
                    Console.WriteLine("stage: " + stage);
                    if (stage == SerialStage.GreetHost)
                    {
                        // First, send a message to the device
                        serial.Write("010");
                        stage = SerialStage.GreetDevice;
                    }
                    else if (stage == SerialStage.GreetDevice)
                    {
                        // Next, wait for a response. If one isn't recieved in 5 seconds, restart.
                        var check = Task.Run(() => SerialResponse("101"));
                        stage = check.Wait(TimeSpan.FromSeconds(5)) ? SerialStage.RecvParts : SerialStage.GreetHost;
                    }
                    else if (stage == SerialStage.RecvParts)
                    {
                        lock (_compMutex)
                        {
                            serial.Write($"{cpuName}|{gpuName}|{ramTotal}GB|");
                        }
                        stage = SerialStage.RecvConfirm;
                    }
                    else if (stage == SerialStage.RecvConfirm)
                    {
                        var check = Task.Run(() => SerialResponse("222"));
                        stage = check.Wait(TimeSpan.FromSeconds(5)) ? SerialStage.ContStats : SerialStage.GreetHost;
                    }
                    else if (stage == SerialStage.ContStats)
                    {
                        lock (_compMutex)
                        {
                            serial.Write($"{cpuFreq}|{cpuTemp}|{cpuLoad}|{ramUsed}|{gpuTemp}|" +
                                $"{gpuCoreClock}|{gpuCoreLoad}|{gpuVramClock}|{gpuVramLoad}|");
                        }
                        stage = SerialStage.ContConfrim;
                    }
                    else if (stage == SerialStage.ContConfrim)
                    {
                        // wait for response "333" for 5 seconds, if nothing then restart
                        var check = Task.Run(() => SerialResponse("333"));
                        stage = check.Wait(TimeSpan.FromSeconds(5)) ? SerialStage.ContStats : SerialStage.GreetHost;
                    }
                }

                // Sleep for a moment, we don't need to spam the serial pipe.
                Thread.Sleep(100);
            }
        }

        private void SerialResponse(string expected)
        {
            while (true)
            {
                if (serial.BytesToRead > 0) {

                    string s = serial.ReadExisting();
                    Console.WriteLine("serial: " + s);
                    if (s == expected)
                        return;
                }
                Thread.Sleep(25);
            }
        }

        private void updatetick_Tick(object sender, EventArgs e)
        {   
            // Update Computer object, every so often.
            UpdateComp();
        }

        private void MainForm_Resize(object sender, EventArgs e)
        {
            // Don't close the whole app, but just hide the window, and go back to the systray.
            if (WindowState == FormWindowState.Minimized)
            {
                Hide();
                trayIcon.Visible = true;
            }
        }

        private void exitToolStripMenuItem_Click(object sender, EventArgs e)
        {
            // Break down serial comms
            _continueComms = false;
            comms.Join();
            if (serial.IsOpen) serial.Close();

            // Close the program
            _closing = true;
            Close();
            Environment.Exit(0);
        }

        private void showToolStripMenuItem_Click(object sender, EventArgs e)
        {
            // Show the GUI window! We don't need to stick around in the systray now.
            Show();
            trayIcon.Visible = false;
        }

        private void MainForm_FormClosing(object sender, FormClosingEventArgs e)
        {
            // Program won't close unless closed from the systray
            if (e.CloseReason == CloseReason.UserClosing && !_closing)
            {
                Hide();
                trayIcon.Visible = true;
                e.Cancel = true;
            }
        }

        private void trayMenu_Opening(object sender, CancelEventArgs e)
        {
            // Manage items by changes in the list.
            var coms = new List<string>();

            // Window's BS way of getting the friendly name of all the serial devices.
            var searcher = new ManagementObjectSearcher(
                "ROOT\\CIMV2",
                "SELECT * FROM Win32_PnPEntity WHERE" +
                " ClassGuid='{4d36e978-e325-11ce-bfc1-08002be10318}'" +
                " AND DeviceID LIKE '%VID_1209&PID_F20A%'"
            );
            foreach (var o in searcher.Get())
            {
                var m = (ManagementObject) o;
                coms.Add(m["Name"].ToString());
            }

            // If any of the items no longer exist (COM's disconnected) or the options are satic, remove them from the list.
            for (var i = trayMenu.Items.Count-1; i >= 0; i--)
            {
                var s = trayMenu.Items[i];
                if ((string)s.Tag != "important" && coms.IndexOf(s.Text) == -1)
                    trayMenu.Items.Remove(s);
            }

            // Go through each COM, check if it exists in the list, and if not add it.
            foreach (var s in coms)
            {
                var item = new ToolStripMenuItem {Text = s};
                var shouldadd = true;
                for (var i = trayMenu.Items.Count - 1; i >= 0; i--)
                {
                    var t = trayMenu.Items[i];
                    if (t.Text == s)
                        shouldadd = false;
                }
                if (shouldadd)
                    trayMenu.Items.Insert(1, item);
            }
        }

        private void trayMenu_ItemClicked(object sender, ToolStripItemClickedEventArgs e)
        {
            // If the user clicks on a static element, leave it alone.
            if ((string)e.ClickedItem.Tag == "important") {
                return;
            }

            // Grab the COM name from the list via regex
            var comchosen = e.ClickedItem.Text;
            var match = Regex.Match(comchosen, @"[a-zA-Z\ ]+\((COM[0-9]+)\)", RegexOptions.IgnoreCase);

            // Uh oh!
            if (!match.Success)
            {
                MessageBox.Show(
                    PopupNotActualSerial,
                    PopupFailedToOpen,
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Error
                );
                return;
            }

            // Temporary variable for the checked item.
            bool shouldCheck = !((ToolStripMenuItem)e.ClickedItem).Checked;
            // Disable the rest of the checkmarks on the items.
            foreach (ToolStripItem t in trayMenu.Items)
                ((ToolStripMenuItem)t).Checked = false;
            // Change the selected item's checkmark
            ((ToolStripMenuItem)e.ClickedItem).Checked = shouldCheck;
            _sendmessage = shouldCheck;

            // Time to connect with serial!
            try
            {
                lock (serial)
                {
                    if (serial.IsOpen)
                        serial.Close();

                    serial.PortName = match.Groups[1].Value;

                    stage = SerialStage.GreetHost;

                    if (shouldCheck)
                        serial.Open();
                }
            }
            catch (System.UnauthorizedAccessException)
            {
                MessageBox.Show(
                    PopupSerialInUse,
                    PopupFailedToOpen,
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Error
                );
            }
            catch (System.IO.IOException)
            {
                MessageBox.Show(
                    PopupSerialFailed,
                    PopupFailedToOpen,
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Error
                );
            }
        }
    }

    // Visitor, for grabbing/updating the values of all the hardware we check.
    public class UpdateVisitor : IVisitor
    {
        public void VisitComputer(IComputer computer)
        {
            computer.Traverse(this);
        }
        public void VisitHardware(IHardware hardware)
        {
            hardware.Update();
            foreach (var subHardware in hardware.SubHardware) subHardware.Accept(this);
        }
        public void VisitSensor(ISensor sensor) { }
        public void VisitParameter(IParameter parameter) { }
    }
}