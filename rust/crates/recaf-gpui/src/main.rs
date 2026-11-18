//! Recaf Rust — a GPUI-based desktop front end for the Rust side of
//! Recaf-Rust.
//!
//! Launch it with no arguments to inspect an embedded demo class file, or
//! pass the path of any `.class` file as the first argument:
//!
//! ```text
//! recaf-rust                                   # embedded demo class
//! recaf-rust path/to/File.class                # any real class file
//! ```

use gpui::{
    actions, div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Menu, MenuItem,
    SharedString, TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use recaf_classfile::{class_flags_to_names, ClassFile, ConstantPoolEntry};

mod sample;

actions!(recaf_rust, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

// Dark theme palette.
const BG: u32 = 0x14161b;
const PANEL: u32 = 0x1d2129;
const ROW: u32 = 0x232833;
const BORDER: u32 = 0x2a3040;
const TEXT: u32 = 0xe6e9ef;
const MUTED: u32 = 0x8b93a7;
const ACCENT: u32 = 0x5eead4;
const ACCENT_BG: u32 = 0x12312b;

/// (chip background, chip text) color pair for a constant pool tag group.
fn tag_colors(entry: &ConstantPoolEntry) -> (u32, u32) {
    match entry {
        ConstantPoolEntry::Utf8(_) | ConstantPoolEntry::String { .. } => (0x172a3f, 0x7cc4ff),
        ConstantPoolEntry::Class { .. } => (0x16302a, 0x5eead4),
        ConstantPoolEntry::Fieldref { .. }
        | ConstantPoolEntry::Methodref { .. }
        | ConstantPoolEntry::InterfaceMethodref { .. } => (0x33270f, 0xfbbf24),
        ConstantPoolEntry::NameAndType { .. } | ConstantPoolEntry::MethodType { .. } => {
            (0x241a38, 0xc4b5fd)
        }
        ConstantPoolEntry::Integer(_)
        | ConstantPoolEntry::Long(_)
        | ConstantPoolEntry::Float(_)
        | ConstantPoolEntry::Double(_) => (0x351523, 0xf9a8d4),
        ConstantPoolEntry::MethodHandle { .. }
        | ConstantPoolEntry::Dynamic { .. }
        | ConstantPoolEntry::InvokeDynamic { .. } => (0x341322, 0xfb7185),
        ConstantPoolEntry::Module { .. } | ConstantPoolEntry::Package { .. } => {
            (0x0f2f2b, 0x2dd4bf)
        }
    }
}

struct PoolRow {
    index: u16,
    tag: &'static str,
    description: String,
    chip_bg: u32,
    chip_text: u32,
}

/// The root window view.
struct RecafRust {
    class: ClassFile,
    source_label: SharedString,
}

fn info_row(label: &'static str, value: String) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().text_xs().text_color(rgb(MUTED)).child(label))
        .child(div().truncate().text_color(rgb(TEXT)).child(value))
}

fn pool_row(row: &PoolRow) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(rgb(ROW))
        .border_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .w(px(36.))
                .text_xs()
                .text_color(rgb(MUTED))
                .child(format!("#{:<3}", row.index)),
        )
        .child(
            div()
                .px_2()
                .h(px(18.))
                .flex()
                .items_center()
                .rounded_md()
                .bg(rgb(row.chip_bg))
                .text_xs()
                .text_color(rgb(row.chip_text))
                .child(row.tag),
        )
        .child(
            div()
                .flex_1()
                .truncate()
                .text_color(rgb(TEXT))
                .child(row.description.clone()),
        )
}

impl Render for RecafRust {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let pool = &self.class.constant_pool;
        let version_label = format!(
            "{} — class file {}",
            self.class.java_version(),
            self.class.class_file_version()
        );
        let access_label = class_flags_to_names(self.class.access_flags).join(" ");
        let interfaces_label = if self.class.interfaces.is_empty() {
            "(none)".to_string()
        } else {
            format!(
                "{}: {}",
                self.class.interfaces.len(),
                self.class.interfaces.join(", ")
            )
        };
        let flags_label = format!(
            "Field attributes: {} · Method attributes: {} · Class attributes: {}",
            self.class
                .fields
                .iter()
                .map(|f| f.attribute_names.len())
                .sum::<usize>(),
            self.class
                .methods
                .iter()
                .map(|m| m.attribute_names.len())
                .sum::<usize>(),
            self.class.attribute_names.len(),
        );

        let pool_rows: Vec<PoolRow> = pool
            .entries()
            .map(|(index, entry)| {
                let (chip_bg, chip_text) = tag_colors(entry);
                PoolRow {
                    index,
                    tag: entry.tag_name(),
                    description: pool.describe(index).unwrap_or_else(|| entry.to_string()),
                    chip_bg,
                    chip_text,
                }
            })
            .collect();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG))
            .text_color(rgb(TEXT))
            .text_sm()
            // Header bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .bg(rgb(PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .text_xl()
                                    .child(div().text_color(rgb(ACCENT)).child("Recaf"))
                                    .child(div().child("Rust")),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(MUTED))
                                    .child("Java class file inspector — Rust + GPUI"),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(ACCENT_BG))
                            .text_xs()
                            .text_color(rgb(ACCENT))
                            .child(version_label),
                    ),
            )
            // Body: info card + constant pool panel
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .gap_3()
                    .p_3()
                    .child(
                        div()
                            .w(px(300.))
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_3()
                            .bg(rgb(PANEL))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .rounded_md()
                            .child(
                                div()
                                    .text_color(rgb(ACCENT))
                                    .child(format!("{}", self.class.this_class)),
                            )
                            .child(info_row("Magic", "0xCAFEBABE (verified)".to_string()))
                            .child(info_row("Access flags", access_label))
                            .child(info_row(
                                "Super class",
                                self.class
                                    .super_class
                                    .clone()
                                    .unwrap_or_else(|| "(none)".to_string()),
                            ))
                            .child(info_row("Interfaces", interfaces_label))
                            .child(info_row(
                                "Members",
                                format!(
                                    "{} fields · {} methods",
                                    self.class.fields.len(),
                                    self.class.methods.len()
                                ),
                            ))
                            .child(info_row("Attributes", flags_label)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_3()
                            .bg(rgb(PANEL))
                            .border_1()
                            .border_color(rgb(BORDER))
                            .rounded_md()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .justify_between()
                                    .child(div().text_color(rgb(MUTED)).child("Constant pool"))
                                    .child(div().text_xs().text_color(rgb(MUTED)).child(format!(
                                        "{} entries (count {})",
                                        pool.valid_count(),
                                        pool.count()
                                    ))),
                            )
                            .child(
                                div().flex_1().id("constant-pool").overflow_scroll().child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .children(pool_rows.iter().map(pool_row)),
                                ),
                            ),
                    ),
            )
            // Footer status bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(rgb(PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child(div().flex_1().truncate().child(self.source_label.clone()))
                    .child(div().child("parsed by recaf-classfile (Rust)")),
            )
    }
}

/// Load the class to show: first CLI argument if possible, otherwise the
/// embedded demo class.
fn load_class() -> (ClassFile, SharedString) {
    match std::env::args().nth(1) {
        Some(path) => match std::fs::read(&path) {
            Ok(bytes) => match ClassFile::parse(&bytes) {
                Ok(class) => (class, path.into()),
                Err(error) => (
                    sample::build_demo_class_file(),
                    format!("embedded sample — could not parse {path}: {error}").into(),
                ),
            },
            Err(error) => (
                sample::build_demo_class_file(),
                format!("embedded sample — could not read {path}: {error}").into(),
            ),
        },
        None => (
            sample::build_demo_class_file(),
            "embedded sample — pass a .class file path as the first argument".into(),
        ),
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.on_action(quit);
        cx.set_menus(vec![Menu {
            name: "Recaf Rust".into(),
            items: vec![MenuItem::separator(), MenuItem::action("Quit", Quit)],
        }]);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let (class, source_label) = load_class();
        let bounds = Bounds::centered(None, size(px(980.), px(660.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Recaf Rust — Java class file inspector".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| RecafRust {
                    class,
                    source_label,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
