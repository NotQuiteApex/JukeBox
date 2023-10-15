using System.ComponentModel;
using System.Reflection;
using System.Windows.Forms;

namespace JukeBoxDesktop {
    partial class MainForm
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.components = new Container();
            ComponentResourceManager resources = new ComponentResourceManager(typeof(MainForm));
            this.updatetick = new Timer(this.components);
            this.titleText = new Label();
            this.cpuGroup = new GroupBox();
            this.labelCpuLoad = new Label();
            this.labelCpuTemp = new Label();
            this.labelCpuFreq = new Label();
            this.labelCpuName = new Label();
            this.gpuGroup = new GroupBox();
            this.labelGpuVramLoad = new Label();
            this.labelGpuVramClock = new Label();
            this.labelGpuCoreLoad = new Label();
            this.labelGpuTemp = new Label();
            this.labelGpuCoreClock = new Label();
            this.labelGpuName = new Label();
            this.ramGroup = new GroupBox();
            this.labelRamTotal = new Label();
            this.labelRamUsed = new Label();
            this.trayIcon = new NotifyIcon(this.components);
            this.trayMenu = new ContextMenuStrip(this.components);
            this.maxStatsToolStripMenuItem = new ToolStripMenuItem();
            this.toolStripSeparator1 = new ToolStripSeparator();
            this.toolStripMenuItem2 = new ToolStripSeparator();
            this.showToolStripMenuItem = new ToolStripMenuItem();
            this.toolStripMenuItem1 = new ToolStripSeparator();
            this.exitToolStripMenuItem = new ToolStripMenuItem();
            this.cpuGroup.SuspendLayout();
            this.gpuGroup.SuspendLayout();
            this.ramGroup.SuspendLayout();
            this.trayMenu.SuspendLayout();
            this.SuspendLayout();
            // 
            // updatetick
            // 
            this.updatetick.Enabled = true;
            this.updatetick.Interval = 1000;
            this.updatetick.Tick += new System.EventHandler(this.updatetick_Tick);
            // 
            // titleText
            // 
            this.titleText.AutoSize = true;
            this.titleText.Location = new System.Drawing.Point(12, 9);
            this.titleText.Name = "titleText";
            this.titleText.Size = new System.Drawing.Size(95, 13);
            this.titleText.TabIndex = 0;
            this.titleText.Text = "MaxStats - Testing";
            // 
            // cpuGroup
            // 
            this.cpuGroup.Controls.Add(this.labelCpuLoad);
            this.cpuGroup.Controls.Add(this.labelCpuTemp);
            this.cpuGroup.Controls.Add(this.labelCpuFreq);
            this.cpuGroup.Controls.Add(this.labelCpuName);
            this.cpuGroup.Location = new System.Drawing.Point(12, 25);
            this.cpuGroup.Name = "cpuGroup";
            this.cpuGroup.Size = new System.Drawing.Size(223, 76);
            this.cpuGroup.TabIndex = 1;
            this.cpuGroup.TabStop = false;
            this.cpuGroup.Text = "CPU";
            // 
            // labelCpuLoad
            // 
            this.labelCpuLoad.AutoSize = true;
            this.labelCpuLoad.Location = new System.Drawing.Point(6, 55);
            this.labelCpuLoad.Name = "labelCpuLoad";
            this.labelCpuLoad.Size = new System.Drawing.Size(34, 13);
            this.labelCpuLoad.TabIndex = 3;
            this.labelCpuLoad.Text = "Load:";
            // 
            // labelCpuTemp
            // 
            this.labelCpuTemp.AutoSize = true;
            this.labelCpuTemp.Location = new System.Drawing.Point(6, 42);
            this.labelCpuTemp.Name = "labelCpuTemp";
            this.labelCpuTemp.Size = new System.Drawing.Size(37, 13);
            this.labelCpuTemp.TabIndex = 2;
            this.labelCpuTemp.Text = "Temp:";
            // 
            // labelCpuFreq
            // 
            this.labelCpuFreq.AutoSize = true;
            this.labelCpuFreq.Location = new System.Drawing.Point(6, 29);
            this.labelCpuFreq.Name = "labelCpuFreq";
            this.labelCpuFreq.Size = new System.Drawing.Size(31, 13);
            this.labelCpuFreq.TabIndex = 1;
            this.labelCpuFreq.Text = "Freq:";
            // 
            // labelCpuName
            // 
            this.labelCpuName.AutoSize = true;
            this.labelCpuName.Location = new System.Drawing.Point(6, 16);
            this.labelCpuName.Name = "labelCpuName";
            this.labelCpuName.Size = new System.Drawing.Size(41, 13);
            this.labelCpuName.TabIndex = 0;
            this.labelCpuName.Text = "Name: ";
            // 
            // gpuGroup
            // 
            this.gpuGroup.Controls.Add(this.labelGpuVramLoad);
            this.gpuGroup.Controls.Add(this.labelGpuVramClock);
            this.gpuGroup.Controls.Add(this.labelGpuCoreLoad);
            this.gpuGroup.Controls.Add(this.labelGpuTemp);
            this.gpuGroup.Controls.Add(this.labelGpuCoreClock);
            this.gpuGroup.Controls.Add(this.labelGpuName);
            this.gpuGroup.Location = new System.Drawing.Point(12, 107);
            this.gpuGroup.Name = "gpuGroup";
            this.gpuGroup.Size = new System.Drawing.Size(223, 100);
            this.gpuGroup.TabIndex = 4;
            this.gpuGroup.TabStop = false;
            this.gpuGroup.Text = "GPU";
            // 
            // labelGpuVramLoad
            // 
            this.labelGpuVramLoad.AutoSize = true;
            this.labelGpuVramLoad.Location = new System.Drawing.Point(6, 55);
            this.labelGpuVramLoad.Name = "labelGpuVramLoad";
            this.labelGpuVramLoad.Size = new System.Drawing.Size(68, 13);
            this.labelGpuVramLoad.TabIndex = 5;
            this.labelGpuVramLoad.Text = "VRAM Load:";
            // 
            // labelGpuVramClock
            // 
            this.labelGpuVramClock.AutoSize = true;
            this.labelGpuVramClock.Location = new System.Drawing.Point(6, 68);
            this.labelGpuVramClock.Name = "labelGpuVramClock";
            this.labelGpuVramClock.Size = new System.Drawing.Size(71, 13);
            this.labelGpuVramClock.TabIndex = 4;
            this.labelGpuVramClock.Text = "VRAM Clock:";
            // 
            // labelGpuCoreLoad
            // 
            this.labelGpuCoreLoad.AutoSize = true;
            this.labelGpuCoreLoad.Location = new System.Drawing.Point(6, 29);
            this.labelGpuCoreLoad.Name = "labelGpuCoreLoad";
            this.labelGpuCoreLoad.Size = new System.Drawing.Size(59, 13);
            this.labelGpuCoreLoad.TabIndex = 3;
            this.labelGpuCoreLoad.Text = "Core Load:";
            // 
            // labelGpuTemp
            // 
            this.labelGpuTemp.AutoSize = true;
            this.labelGpuTemp.Location = new System.Drawing.Point(6, 81);
            this.labelGpuTemp.Name = "labelGpuTemp";
            this.labelGpuTemp.Size = new System.Drawing.Size(37, 13);
            this.labelGpuTemp.TabIndex = 2;
            this.labelGpuTemp.Text = "Temp:";
            // 
            // labelGpuCoreClock
            // 
            this.labelGpuCoreClock.AutoSize = true;
            this.labelGpuCoreClock.Location = new System.Drawing.Point(6, 42);
            this.labelGpuCoreClock.Name = "labelGpuCoreClock";
            this.labelGpuCoreClock.Size = new System.Drawing.Size(62, 13);
            this.labelGpuCoreClock.TabIndex = 1;
            this.labelGpuCoreClock.Text = "Core Clock:";
            // 
            // labelGpuName
            // 
            this.labelGpuName.AutoSize = true;
            this.labelGpuName.Location = new System.Drawing.Point(6, 16);
            this.labelGpuName.Name = "labelGpuName";
            this.labelGpuName.Size = new System.Drawing.Size(41, 13);
            this.labelGpuName.TabIndex = 0;
            this.labelGpuName.Text = "Name: ";
            // 
            // ramGroup
            // 
            this.ramGroup.Controls.Add(this.labelRamTotal);
            this.ramGroup.Controls.Add(this.labelRamUsed);
            this.ramGroup.Location = new System.Drawing.Point(12, 213);
            this.ramGroup.Name = "ramGroup";
            this.ramGroup.Size = new System.Drawing.Size(223, 47);
            this.ramGroup.TabIndex = 5;
            this.ramGroup.TabStop = false;
            this.ramGroup.Text = "RAM";
            // 
            // labelRamTotal
            // 
            this.labelRamTotal.AutoSize = true;
            this.labelRamTotal.Location = new System.Drawing.Point(6, 29);
            this.labelRamTotal.Name = "labelRamTotal";
            this.labelRamTotal.Size = new System.Drawing.Size(34, 13);
            this.labelRamTotal.TabIndex = 4;
            this.labelRamTotal.Text = "Total:";
            // 
            // labelRamUsed
            // 
            this.labelRamUsed.AutoSize = true;
            this.labelRamUsed.Location = new System.Drawing.Point(6, 16);
            this.labelRamUsed.Name = "labelRamUsed";
            this.labelRamUsed.Size = new System.Drawing.Size(35, 13);
            this.labelRamUsed.TabIndex = 1;
            this.labelRamUsed.Text = "Used:";
            // 
            // trayIcon
            // 
            this.trayIcon.ContextMenuStrip = this.trayMenu;
            this.trayIcon.Icon = System.Drawing.Icon.ExtractAssociatedIcon(Assembly.GetExecutingAssembly().Location);
            this.trayIcon.Text = "MaxStats";
            this.trayIcon.Visible = true;
            // 
            // trayMenu
            // 
            this.trayMenu.Items.AddRange(new ToolStripItem[] {
            this.maxStatsToolStripMenuItem,
            this.toolStripSeparator1,
            this.toolStripMenuItem2,
            this.showToolStripMenuItem,
            this.toolStripMenuItem1,
            this.exitToolStripMenuItem});
            this.trayMenu.Name = "trayMenu";
            this.trayMenu.Size = new System.Drawing.Size(131, 88);
            this.trayMenu.Opening += new CancelEventHandler(this.trayMenu_Opening);
            this.trayMenu.ItemClicked += new ToolStripItemClickedEventHandler(this.trayMenu_ItemClicked);
            // 
            // maxStatsToolStripMenuItem
            // 
            this.maxStatsToolStripMenuItem.Font = new System.Drawing.Font("Segoe UI", 9F, ((System.Drawing.FontStyle)((System.Drawing.FontStyle.Bold | System.Drawing.FontStyle.Underline))), System.Drawing.GraphicsUnit.Point, ((byte)(0)));
            this.maxStatsToolStripMenuItem.Name = "maxStatsToolStripMenuItem";
            this.maxStatsToolStripMenuItem.Size = new System.Drawing.Size(130, 22);
            this.maxStatsToolStripMenuItem.Tag = "important";
            this.maxStatsToolStripMenuItem.Text = "MaxStats";
            // 
            // toolStripSeparator1
            // 
            this.toolStripSeparator1.Name = "toolStripSeparator1";
            this.toolStripSeparator1.Size = new System.Drawing.Size(127, 6);
            // 
            // toolStripMenuItem2
            // 
            this.toolStripMenuItem2.Name = "toolStripMenuItem2";
            this.toolStripMenuItem2.Size = new System.Drawing.Size(127, 6);
            // 
            // showToolStripMenuItem
            // 
            this.showToolStripMenuItem.Name = "showToolStripMenuItem";
            this.showToolStripMenuItem.Size = new System.Drawing.Size(130, 22);
            this.showToolStripMenuItem.Tag = "important";
            this.showToolStripMenuItem.Text = "Show stats";
            this.showToolStripMenuItem.Click += new System.EventHandler(this.showToolStripMenuItem_Click);
            // 
            // toolStripMenuItem1
            // 
            this.toolStripMenuItem1.Name = "toolStripMenuItem1";
            this.toolStripMenuItem1.Size = new System.Drawing.Size(127, 6);
            // 
            // exitToolStripMenuItem
            // 
            this.exitToolStripMenuItem.Name = "exitToolStripMenuItem";
            this.exitToolStripMenuItem.Size = new System.Drawing.Size(130, 22);
            this.exitToolStripMenuItem.Tag = "important";
            this.exitToolStripMenuItem.Text = "Exit";
            this.exitToolStripMenuItem.Click += new System.EventHandler(this.exitToolStripMenuItem_Click);
            // 
            // MainForm
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(245, 271);
            this.Controls.Add(this.ramGroup);
            this.Controls.Add(this.gpuGroup);
            this.Controls.Add(this.cpuGroup);
            this.Controls.Add(this.titleText);
            this.FormBorderStyle = FormBorderStyle.FixedSingle;
            this.Icon = System.Drawing.Icon.ExtractAssociatedIcon(Assembly.GetExecutingAssembly().Location);
            this.MaximizeBox = false;
            this.Name = "MainForm";
            this.SizeGripStyle = SizeGripStyle.Show;
            this.Text = "MaxStats";
            this.FormClosing += new FormClosingEventHandler(this.MainForm_FormClosing);
            this.Resize += new System.EventHandler(this.MainForm_Resize);
            this.cpuGroup.ResumeLayout(false);
            this.cpuGroup.PerformLayout();
            this.gpuGroup.ResumeLayout(false);
            this.gpuGroup.PerformLayout();
            this.ramGroup.ResumeLayout(false);
            this.ramGroup.PerformLayout();
            this.trayMenu.ResumeLayout(false);
            this.ResumeLayout(false);
            this.PerformLayout();

        }

        private Timer updatetick;
        private Label titleText;
        private GroupBox cpuGroup;
        private Label labelCpuFreq;
        private Label labelCpuName;
        private Label labelCpuTemp;
        private Label labelCpuLoad;
        private GroupBox gpuGroup;
        private Label labelGpuVramClock;
        private Label labelGpuCoreLoad;
        private Label labelGpuTemp;
        private Label labelGpuCoreClock;
        private Label labelGpuName;
        private GroupBox ramGroup;
        private Label labelRamTotal;
        private Label labelRamUsed;
        private Label labelGpuVramLoad;
        private NotifyIcon trayIcon;
        private ContextMenuStrip trayMenu;
        private ToolStripMenuItem showToolStripMenuItem;
        private ToolStripMenuItem exitToolStripMenuItem;
        private ToolStripSeparator toolStripSeparator1;
        private ToolStripSeparator toolStripMenuItem1;
        private ToolStripMenuItem maxStatsToolStripMenuItem;
        private ToolStripSeparator toolStripMenuItem2;
    }
}
