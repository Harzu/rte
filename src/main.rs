mod editor;

use editor::Editor;

fn main() -> Result<(), std::io::Error> {
    let mut editor = Editor::new()?;
    editor.run()
}
