use adw::prelude::*;
use relm4::prelude::*;
use relm4::factory::FactoryVecDeque;
use sysinfo::{System, Networks, Disks, Signal, Pid};
use std::time::Duration;
use std::collections::VecDeque;
use relm4::gtk::glib;
use std::fs;
use std::ffi::CStr;
use libc::{uname, utsname};

const MAX_HISTORY: usize = 60;

struct ProcessModel {
    pid: Pid,
    name: String,
    cpu: f32,
    mem: u64,
}

#[derive(Debug)]
enum ProcessInput { Kill }

#[relm4::factory(pub)]
impl FactoryComponent for ProcessModel {
    type Init = (Pid, String, f32, u64);
    type Input = ProcessInput;
    type Output = Pid;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            set_title: &self.name,
            #[watch]
            set_subtitle: &format!("PID: {} | CPU: {:.1}% | MEM: {:.1} MB", self.pid, self.cpu, self.mem as f64 / 1_000_000.0),
            add_suffix = &gtk::Button {
                set_icon_name: "edit-delete-symbolic",
                add_css_class: "destructive-action",
                connect_clicked => ProcessInput::Kill,
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { pid: init.0, name: init.1, cpu: init.2, mem: init.3 }
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            ProcessInput::Kill => { let _ = sender.output(self.pid); }
        }
    }
}

struct CoreModel { index: usize, history: VecDeque<f64> }

#[relm4::factory(pub)]
impl FactoryComponent for CoreModel {
    type Init = usize; type Input = f64; type Output = (); type CommandOutput = (); type ParentWidget = gtk::FlowBox;
    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical, set_spacing: 4,
            append = &gtk::Label { set_label: &format!("Core {}", self.index), set_halign: gtk::Align::Start, add_css_class: "caption" },
            #[name = "graph"]
            append = &gtk::DrawingArea {
                set_height_request: 50, set_width_request: 100,
                #[watch]
                set_draw_func: {
                    let history = self.history.clone();
                    move |_, cr, w, h| draw_graph(cr, w as f64, h as f64, &history, (0.2, 0.6, 1.0), true)
                }
            }
        }
    }
    fn init_model(index: Self::Init, _: &DynamicIndex, _: FactorySender<Self>) -> Self {
        Self { index, history: VecDeque::from(vec![0.0; MAX_HISTORY]) }
    }
    fn update(&mut self, usage: Self::Input, _: FactorySender<Self>) {
        self.history.pop_front(); self.history.push_back(usage);
    }
}

struct AppWidgets {
    main_stack: gtk::Stack,
    ram_label: gtk::Label,
    net_down_label: gtk::Label,
    net_up_label: gtk::Label,
    disk_label: gtk::Label,
    cpu_label: gtk::Label,
    ram_graph: gtk::DrawingArea,
    net_down_graph: gtk::DrawingArea,
    net_up_graph: gtk::DrawingArea,
    disk_graph: gtk::DrawingArea,
    cpu_graph: gtk::DrawingArea,
    uptime_label: gtk::Label,
    cpu_freq_label: gtk::Label,
    os_label: gtk::Label,
    kernel_label: gtk::Label,
}

struct AppModel {
    sys: System,
    networks: Networks,
    disks: Disks,
    ram_history: VecDeque<f64>,
    net_down_history: VecDeque<f64>,
    net_up_history: VecDeque<f64>,
    disk_history: VecDeque<f64>,
    cpu_history: VecDeque<f64>,
    cpu_cores: FactoryVecDeque<CoreModel>,
    processes: FactoryVecDeque<ProcessModel>,
    widgets: AppWidgets,
}

#[derive(Debug)]
enum AppMsg { Tick, SwitchPage(i32), KillProcess(Pid) }

#[relm4::component]
impl Component for AppModel {
    type Init = (); 
    type Input = AppMsg; 
    type Output = (); 
    type CommandOutput = ();

    view! {
        adw::ApplicationWindow {}
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let sys = System::new_all();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        let builder = gtk::Builder::from_file("src/main.ui");
        
        // Error handling for UI objects
        let main_view: adw::NavigationSplitView = builder.object("main_split_view").expect("Could not find main_split_view in UI file");
        root.set_content(Some(&main_view));
        root.set_default_width(1000);
        root.set_default_height(800);
        root.set_title(Some("System Manager"));

        let main_stack: gtk::Stack = builder.object("main_stack").expect("Could not find main_stack");
        let sidebar_list: gtk::ListBox = builder.object("sidebar_list").expect("Could not find sidebar_list");
        
        let s = sender.clone();
        sidebar_list.connect_row_activated(move |_, row| {
            s.input(AppMsg::SwitchPage(row.index()));
        });

        let cpu_cores = FactoryVecDeque::builder()
            .launch(builder.object("cpu_flow_box").expect("Could not find cpu_flow_box"))
            .detach();

        let processes = FactoryVecDeque::builder()
            .launch(builder.object("process_list").expect("Could not find process_list"))
            .forward(sender.input_sender(), AppMsg::KillProcess);
        
        let widgets = AppWidgets {
            main_stack,
            ram_label: builder.object("ram_label").unwrap(),
            net_down_label: builder.object("net_down_label").unwrap(),
            net_up_label: builder.object("net_up_label").unwrap(),
            disk_label: builder.object("disk_label").unwrap(),
            cpu_label: builder.object("cpu_label").unwrap(),
            ram_graph: builder.object("ram_graph").unwrap(),
            net_down_graph: builder.object("net_down_graph").unwrap(),
            net_up_graph: builder.object("net_up_graph").unwrap(),
            disk_graph: builder.object("disk_graph").unwrap(),
            cpu_graph: builder.object("cpu_graph").unwrap(),
            uptime_label: builder.object("uptime_label").unwrap(),
            cpu_freq_label: builder.object("cpu_freq_label").unwrap(),
            os_label: builder.object("os_label").unwrap(),
            kernel_label: builder.object("kernel_label").unwrap(),
        };

        let mut model = AppModel {
            sys, networks, disks,
            ram_history: VecDeque::from(vec![0.0; MAX_HISTORY]),
            net_down_history: VecDeque::from(vec![0.0; MAX_HISTORY]),
            net_up_history: VecDeque::from(vec![0.0; MAX_HISTORY]),
            disk_history: VecDeque::from(vec![0.0; MAX_HISTORY]),
            cpu_history: VecDeque::from(vec![0.0; MAX_HISTORY]),
            cpu_cores, processes,
            widgets,
        };

        for i in 0..model.sys.cpus().len() { 
            model.cpu_cores.guard().push_back(i); 
        }

        let s_tick = sender.clone();
        glib::timeout_add_local(Duration::from_secs(1), move || { 
            s_tick.input(AppMsg::Tick); 
            glib::ControlFlow::Continue 
        });

        let relm_widgets = view_output!();
        ComponentParts { model, widgets: relm_widgets }
    }

    fn update(&mut self, msg: Self::Input, _: ComponentSender<Self>, _: &Self::Root) {
        match msg {
            AppMsg::SwitchPage(idx) => {
                // The index corresponds to the row order in your sidebar_list
                // Index 0 = Resources, Index 1 = Processes
                let target_page = match idx {
                    0 => "page_resources",
                    1 => "page_processes",
                    _ => "page_resources",
                };

                // Switch the stack view
                self.widgets.main_stack.set_visible_child_name(target_page);

                // FIX: Scaling/Maximizing reset
                // Sometimes GTK needs a hint to recalculate layout after a stack switch
                self.widgets.main_stack.queue_resize();
            }

            AppMsg::KillProcess(pid) => {
                if let Some(process) = self.sys.process(pid) {
                    let _ = process.kill_with(Signal::Term);
                }
            }

            AppMsg::Tick => {
                // 1. Refresh System Data
                self.sys.refresh_all();
                self.networks.refresh(true);
                self.disks.refresh(true);

                // 2. Update CPU Core Factory
                {
                    let core_guard = self.cpu_cores.guard();
                    for (i, cpu) in self.sys.cpus().iter().enumerate() { 
                        core_guard.send(i, cpu.cpu_usage() as f64); 
                    }
                }

                // 3. Update Processes Factory (Top 20)
                {
                    let mut proc_guard = self.processes.guard();
                    proc_guard.clear();
                    let mut procs: Vec<_> = self.sys.processes().values().collect();
                    procs.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
                    for p in procs.iter().take(20) {
                        proc_guard.push_back((
                            p.pid(), 
                            p.name().to_string_lossy().to_string(), 
                            p.cpu_usage(), 
                            p.memory()
                        ));
                    }
                }

                // 4. Update History Buffers (Network/RAM/Disk/CPU)
                let mut total_down = 0.0;
                let mut total_up = 0.0;
                for (_, data) in &self.networks {
                    total_down += data.received() as f64 / 1024.0;
                    total_up += data.transmitted() as f64 / 1024.0;
                }
                self.net_down_history.pop_front(); self.net_down_history.push_back(total_down);
                self.net_up_history.pop_front(); self.net_up_history.push_back(total_up);

                let ram_pct = (self.sys.used_memory() as f64 / self.sys.total_memory() as f64) * 100.0;
                self.ram_history.pop_front(); self.ram_history.push_back(ram_pct);

                let mut disk_pct = 0.0;
                if let Some(disk) = self.disks.iter().next() {
                    let used = disk.total_space() - disk.available_space();
                    disk_pct = (used as f64 / disk.total_space() as f64) * 100.0;
                }
                self.disk_history.pop_front(); self.disk_history.push_back(disk_pct);

                let total_cpu = self.sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / self.sys.cpus().len() as f32;
                self.cpu_history.pop_front(); self.cpu_history.push_back(total_cpu as f64);

                // 5. Update UI Widgets
                let w = &self.widgets;
                w.cpu_label.set_label(&format!("{:.1}%", total_cpu));
                w.net_down_label.set_label(&format!("{:.1} KB/s", total_down));
                w.net_up_label.set_label(&format!("{:.1} KB/s", total_up));
                w.ram_label.set_label(&format!("{:.1} / {:.1} GB", 
                    self.sys.used_memory() as f64 / 1_000_000_000.0,
                    self.sys.total_memory() as f64 / 1_000_000_000.0
                ));
                w.disk_label.set_label(&format!("{:.1}% Used", disk_pct));
                
                // Update system info labels
                if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
                    if let Some(seconds_str) = uptime_str.split_whitespace().next() {
                        if let Ok(seconds) = seconds_str.parse::<f64>() {
                            let hours = (seconds / 3600.0) as u64;
                            let mins = ((seconds % 3600.0) / 60.0) as u64;
                            w.uptime_label.set_label(&format!("{}h {}m", hours, mins));
                        }
                    }
                }
                let freq = self.sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);
                w.cpu_freq_label.set_label(&format!("{} MHz", freq));
                
                let mut uts = unsafe { std::mem::zeroed::<utsname>() };
                if unsafe { uname(&mut uts) } == 0 {
                    let sysname = unsafe { CStr::from_ptr(uts.sysname.as_ptr()) }.to_string_lossy();
                    let release = unsafe { CStr::from_ptr(uts.release.as_ptr()) }.to_string_lossy();
                    w.os_label.set_label(&sysname);
                    w.kernel_label.set_label(&release);
                }
                
                // Redraw graphs with updated history
                let h_cpu = self.cpu_history.clone();
                w.cpu_graph.set_draw_func(move |_, cr, width, height| 
                    draw_graph(cr, width as f64, height as f64, &h_cpu, (0.2, 0.6, 1.0), true));

                let h_down = self.net_down_history.clone();
                w.net_down_graph.set_draw_func(move |_, cr, width, height| 
                    draw_graph(cr, width as f64, height as f64, &h_down, (0.1, 0.6, 0.8), false));

                let h_up = self.net_up_history.clone();
                w.net_up_graph.set_draw_func(move |_, cr, width, height| 
                    draw_graph(cr, width as f64, height as f64, &h_up, (0.8, 0.2, 0.2), false));

                let h_ram = self.ram_history.clone();
                w.ram_graph.set_draw_func(move |_, cr, width, height| 
                    draw_graph(cr, width as f64, height as f64, &h_ram, (0.1, 0.8, 0.4), true));

                let h_disk = self.disk_history.clone();
                w.disk_graph.set_draw_func(move |_, cr, width, height| 
                    draw_graph(cr, width as f64, height as f64, &h_disk, (0.5, 0.2, 0.8), true));
                
                // Trigger the actual draw
                w.cpu_graph.queue_draw();
                w.net_down_graph.queue_draw(); 
                w.net_up_graph.queue_draw();
                w.ram_graph.queue_draw();
                w.disk_graph.queue_draw();
            }
        }
    }
}

fn draw_graph(cr: &gtk::cairo::Context, w: f64, h: f64, data: &VecDeque<f64>, color: (f64, f64, f64), fixed: bool) {
    cr.set_source_rgba(0.5, 0.5, 0.5, 0.15); cr.set_line_width(1.0); cr.set_dash(&[4.0, 4.0], 0.0);
    for i in 1..4 { let y = h * (i as f64 / 4.0); cr.move_to(0.0, y); cr.line_to(w, y); }
    let _ = cr.stroke(); cr.set_dash(&[], 0.0);

    let max = if fixed { 100.0 } else { data.iter().fold(1.0, |a: f64, &b| a.max(b)) };
    let step = w / (MAX_HISTORY - 1) as f64;

    cr.set_source_rgba(color.0, color.1, color.2, 0.1); cr.move_to(0.0, h);
    for (i, &v) in data.iter().enumerate() { cr.line_to(i as f64 * step, h - (v / max * h)); }
    cr.line_to(w, h); let _ = cr.fill();

    cr.set_source_rgb(color.0, color.1, color.2); cr.set_line_width(2.0);
    for (i, &v) in data.iter().enumerate() {
        let (x, y) = (i as f64 * step, h - (v / max * h));
        if i == 0 { cr.move_to(x, y); } else { cr.line_to(x, y); }
    }
    let _ = cr.stroke();
}

fn main() { RelmApp::new("com.coopershaw.system-manager").run::<AppModel>(()); }