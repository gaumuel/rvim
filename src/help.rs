pub const HELP_TEXT: &str = "\
rvim - A minimal Vim-like editor

MODES
  Normal     Default mode. Navigate and manipulate text.
  Insert     Type text freely. Enter with i/a/o/O.
  Visual     Select text. Enter with v.
  Command    Execute commands. Enter with :

NORMAL MODE
  h / Left       Move left
  l / Right      Move right
  j / Down       Move down
  k / Up         Move up
  w              Next word
  b              Previous word
  0              Start of line
  $              End of line
  g              First line
  G              Last line
  i              Insert before cursor
  a              Insert after cursor
  o              New line below
  O              New line above
  x              Delete character
  d              Delete line
  v              Enter Visual mode
  :              Enter Command mode

INSERT MODE
  Esc            Return to Normal mode
  Backspace      Delete character before cursor
  Enter          New line
  Arrow keys     Navigate

VISUAL MODE
  h/j/k/l        Extend selection
  w/b/0/$        Extend selection
  d / x          Delete selection
  y              Yank selection
  Esc            Cancel selection

COMMANDS
  Commands are grouped by prefix for easy discovery.
  Type :f, :b, :t, or :set to see each group in the palette.

  FILE (:f)
    :fw            Save file
    :fw <name>     Save as <name>
    :fq            Close tab (or close help)
    :fq!           Force close tab
    :fx            Save and close tab

  BUFFER (:b)
    :bnew          Create new empty buffer (current tab)
    :buffer <id>   Switch current tab to buffer by ID
    :bk            Kill current buffer
    :bk <id>      Kill buffer by ID
    :bl            List all open buffers

  TAB (:t)
    :tnew              Open new empty tab
    :tnew <file>       Open file in new tab
    :tnext             Switch to next tab
    :tprev             Switch to previous tab
    :tclose            Close current tab

  SETTINGS (:set)
    :set cursorline      Enable cursor line highlight
    :set nocursorline    Disable cursor line highlight

  OTHER
    :help          Show this help
    :e <file>      Open file in new buffer (current tab)

  ALIASES (vim-compatible)
    :w :q :q! :wq :x :gt :gT :kb :ls
    :tabnew :tabnext :tabprev :tabclose

GLOBAL
  Ctrl+C         Force quit all

Press :q to close this help.
";
