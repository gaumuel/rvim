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
  :w             Save file
  :w <name>     Save as <name>
  :q             Close tab (or close help)
  :q!            Force close tab
  :wq / :x      Save and close tab
  :help          Show this help

TABS
  :tabnew            Open new empty tab
  :tabnew <file>     Open file in new tab
  :tabnext / :gt     Switch to next tab
  :tabprev / :gT     Switch to previous tab
  :tabclose          Close current tab

BUFFERS
  :e <file>          Open file in new buffer (current tab)
  :bnew              Create new empty buffer (current tab)
  :buffer <id>       Switch current tab to buffer by ID
  :kb                Kill current buffer (tab stays, switches to another)
  :kb <id>           Kill buffer by ID
  :ls                List all open buffers

  Buffers are shared - multiple tabs can view the same buffer.
  If a buffer crashes, switch to another with :buffer <id>.

SETTINGS
  :set cursorline      Enable cursor line highlight
  :set nocursorline    Disable cursor line highlight

GLOBAL
  Ctrl+C         Force quit all

Press :q to close this help.
";
