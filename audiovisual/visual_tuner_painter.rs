use pitch_detector::{core::NoteName, note::NoteDetectionResult};

/*
Based on a NoteDetectionResult from pitch_detector crate. One future refactor might be to keep an enum of interface elements so their types can be passed around easily and checked, 
and interface elements are structs and keep their offsets and colors stored in there. Would be better when interface gets more complex, overkill for current goals.
      
pub struct NoteDetectionResult 
    /// The predominant frequency detected from a signal.
    pub actual_freq: f64,

    /// The note name of the detected note.
    pub note_name: NoteName,

    /// The expected frequency of the detected note.
    pub note_freq: f64,

    /// The octave of the detected note.
    pub octave: i32,

    /// The degree to which the detected not is in tune, expressed in cents. The absolute maximum `cents_offset` is
    /// 50, since anything larger than 50 would be considered the next or previous note.
    pub cents_offset: f64,

    /// The note name of the note that comes before the detected note. Not commonly used.
    pub previous_note_name: NoteName,

    /// The note name of the note that comes after the detected note. Not commonly used.
    pub next_note_name: NoteName,

    /// A `NoteDetectionResult` will be marked as `in_tune` if the `cents_offset` is less than
    /// [`MAX_CENTS_OFFSET`](crate::core::constants::MAX_CENTS_OFFSET).
    pub in_tune: bool,
*/

struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

/*
Painter keeps some general state and runs the steps to draw layers.
*/
pub struct Painter {
    led_matrix_max_x: usize,
    led_matrix_max_y: usize,
}
impl Painter {
    pub fn new(x: usize, y: usize) -> Self {
        let mut bar_ghosts: Vec<Option<RGB>> = Vec::with_capacity(x*y);
        for _ in 0..x*y {
            bar_ghosts.push(None);
        }

        Painter {
            led_matrix_max_x: x,
            led_matrix_max_y: y,
        }
    }
    pub fn paint(&mut self, note_det_result: &NoteDetectionResult) -> Vec<u8> {
        let detected_note = &note_det_result.note_name;
        let prev_note = &note_det_result.previous_note_name;
        let next_note = &note_det_result.next_note_name;
        let cents_offset = note_det_result.cents_offset;
        let in_tune = note_det_result.in_tune;

        println!("{} {} {} {} {}", &note_det_result.note_name, &note_det_result.previous_note_name, &note_det_result.next_note_name, &note_det_result.cents_offset, &note_det_result.in_tune);

        let blank_canvas = BlankCanvas::new(self.led_matrix_max_x, self.led_matrix_max_y);
        let base_lined = blank_canvas.draw_baseline();
        let detected_line_drawn = base_lined.draw_detected_line(cents_offset);
        let notes_drawn = detected_line_drawn.draw_notes(detected_note, prev_note, next_note, in_tune);

        notes_drawn.output()
    }
}

/*
Typestates:
 - Blank canvas
 - Draw baseline for note (the tune goal frequency)
 - Draw detected line for note (the estimate of the actual frequency being played)
 - Draw note names
*/
struct BlankCanvas {
    max_x: usize,
    max_y: usize,
    color_vec: Vec<RGB>,

    // setting for the line to draw
    base_line_color: RGB
}
struct BaseLined {
    max_x: usize,
    max_y: usize,
    color_vec: Vec<RGB>,

    // settings for the line to draw
    detected_line_color: RGB,
    baseline_row: usize,
}

struct DetectedLineDrawn {
    max_x: usize,
    color_vec: Vec<RGB>,

    // settings for the note to draw
    detected_note_color: RGB,
    in_tune_color: RGB,
    adjacent_note_color: RGB,
    start_row_col_detected: (usize, usize),
    start_row_col_prev: (usize, usize),
    start_row_col_next: (usize, usize)
}

struct NotesDrawn {
    color_vec: Vec<RGB>,
}

impl BlankCanvas {
    pub fn new(max_x: usize, max_y: usize) -> BlankCanvas {
        let mut empty_canvas = Vec::with_capacity(max_x*max_y);
        for _ in 0..(max_x*max_y) {
            empty_canvas.push(RGB{r:1,g:1,b:5});
        }

        BlankCanvas {
            max_x,
            max_y,
            color_vec: empty_canvas,
            base_line_color: RGB{r:255,g:215,b:0}
        }
    }

    fn draw_baseline(mut self) -> BaseLined {
        let baseline_row = (self.max_y as f32 / 2.0).round() as usize; // draw line starting at light 1 in row 17 (index 16), fixed around the center of the vertically placed ledstrip
        let color_vec_index_baseline_row = baseline_row*self.max_x;

        //If baseline is chosen on a serpentine row the for loop needs to be changed (see next state for a line draw loop with serpentine) 
        for i in 0.. self.max_x {
            self.color_vec[i + color_vec_index_baseline_row] = RGB{r: self.base_line_color.r, g:self.base_line_color.g, b: self.base_line_color.b};
        }

        BaseLined {
            detected_line_color: RGB{r:199,g:129,b:19},
            baseline_row,
            max_x: self.max_x,
            max_y: self.max_y,
            color_vec: self.color_vec
        }
    }
}

impl BaseLined {
    fn draw_detected_line(mut self, cents_offset: f64) -> DetectedLineDrawn {      
        // -1 because even number leds with baseline in middle -> max distance is 1 less at one side of the baseline 
        let max_distance = self.max_y - self.baseline_row - 1;

        // draw the line in the positive or negative direction at cents_offset divided by 50
        // because as soon as the offset is more than 50% a new note becomes the baseline
        let offset_distance = (max_distance as f64 * cents_offset / 50.0).round() as i16;
        let draw_row = (self.baseline_row as i16 + offset_distance) as usize;

        for i in 0.. self.max_x {
            let mut x = i;
            if draw_row % 2 == 1{
                x = 7-i; // serpentine row, start drawing at last led and move back
            }

            self.color_vec[x + (draw_row * self.max_x)] = RGB{r: self.detected_line_color.r, g: self.detected_line_color.g, b: self.detected_line_color.b};
        }

        DetectedLineDrawn {
            max_x: self.max_x,
            color_vec: self.color_vec,
            detected_note_color: RGB{r:200, g:0, b: 0},
            in_tune_color: RGB{r:0, g:255, b:0},
            adjacent_note_color: RGB{r:100, g:0, b: 100},
            start_row_col_detected: (14, 1),
            start_row_col_prev: (1, 1),
            start_row_col_next: (25, 1)
        }
    }
}

// this could be greatly expanded upon and make code relating to lines and offsets for GUI elements nicer, but for now this only tracks three guitar note types
enum InterfaceElement {
    DetectedNote,
    PreviousNote,
    NextNote
}

impl DetectedLineDrawn {
    fn draw_notes(mut self, detected_note: &NoteName, prev_note: &NoteName, next_note: &NoteName, in_tune: bool) -> NotesDrawn {
        let graphical_detected_note = GraphicalNote::new(detected_note);
        let graphical_prev_note = GraphicalNote::new(prev_note);
        let graphical_next_note = GraphicalNote::new(next_note);
        
        self.draw_note(graphical_detected_note, InterfaceElement::DetectedNote, in_tune);
        self.draw_note(graphical_prev_note, InterfaceElement::PreviousNote, in_tune);
        self.draw_note(graphical_next_note, InterfaceElement::NextNote, in_tune);
        
        NotesDrawn {
            color_vec: self.color_vec
        }
    }

    // this function doesnt check if the note being drawn bleeds over the end of the row or column, so offsets have to be chosen so that all notes will fit on the ledmatrix
    fn draw_note(&mut self, graphical_note: GraphicalNote, note_type: InterfaceElement, in_tune: bool) {
        let (color, y_offset, x_offset) = match note_type {
            InterfaceElement::DetectedNote => {
                let (y_offset, x_offset) = self.start_row_col_detected;
                let color;
                if in_tune {
                    color = &self.in_tune_color;
                }
                else {
                    color = &self.detected_note_color;
                }
                (color, y_offset, x_offset)
            },
            InterfaceElement::PreviousNote => {
                let (y_offset, x_offset) = self.start_row_col_prev;
                let color = &self.adjacent_note_color;
                (color, y_offset, x_offset)
            },
            InterfaceElement::NextNote => {
                let (y_offset, x_offset) = self.start_row_col_next;
                let color = &self.adjacent_note_color;
                (color, y_offset, x_offset)
            }
        };

        let note_width = graphical_note.matrix[0].len();

        for (i, row) in graphical_note.matrix.iter().rev().enumerate() { //each y of note, start rendering at bottom of note
            for col in 0.. (note_width) { //each x of note
                let mut matrix_x = x_offset + col;
                let matrix_y = y_offset + i;

                if (i+y_offset) % 2 == 1 { // serpentine row
                    matrix_x = self.max_x-1 - x_offset - col;
                }
                if row[col] {
                    self.color_vec[matrix_x + matrix_y*(self.max_x)] = RGB{r: color.r, g: color.g, b: color.b};
                }
            }
        }
    }
}

impl NotesDrawn {
    fn output(self) -> Vec<u8> {
        // led matrix needs a vec of separate GRB values
        self.color_vec.into_iter().flat_map(|rgb| [rgb.g, rgb.r, rgb.b]).collect()
    }
}

/*
Graphical representations of the musical notes that can be found by NoteDetectionResult

Used pixelmatrices so I could visually "draw" the letters with the true values in the definitions below. Flattened later for processing in output.
*/

type PixelMatrix = Vec<Vec<bool>>;

// graphical representation of musical notes, max 2x4 pixels
// drawing NoteDetectionResults from note_detection crate
// also use the NoteName enum from note_detection crate to have consistency for note names

struct GraphicalNote {
    matrix: PixelMatrix,
}

impl GraphicalNote {
    fn new(note: &NoteName) -> Self {
        let matrix = match note {
            NoteName::A => vec![
                vec![false, true, false],
                vec![true, false, true],
                vec![true, true, true],
                vec![true, false,true],
            ],
            NoteName::ASharp => vec![
                vec![false, false, false, false, false, true, false],
                vec![false, false, false, false, true, true, true],
                vec![false, true, false, false, false, true, false],
                vec![true, false, true, false, false, false, false],
                vec![true, true, true, false, false, false, false],
                vec![true, false, true, false, false, false, false],
            ],
            NoteName::B => vec![
                vec![true, true, false],
                vec![true, false, true],
                vec![true, true, false],
                vec![true, false, true],
                vec![true, true, false]
            ],
            NoteName::C => vec![
                vec![false, true, true],
                vec![true, false, false],
                vec![true, false, false],
                vec![false, true, true],
            ],
            NoteName::CSharp => vec![
                vec![false, false, false, false, false, true, false],
                vec![false, true, true, false, true, true, true],
                vec![true, false, false, false, false, true, false],
                vec![true, false, false, false, false, false, false],
                vec![true, false, false, false, false, false, false],
                vec![false, true, true, false, false, false, false],
            ],
            NoteName::D => vec![
                vec![true, true, false],
                vec![true, false, true],
                vec![true, false, true],
                vec![true, true, false],
            ],
            NoteName::DSharp => vec![
                vec![false, false, false, false, false, true, false],
                vec![true, true, false, false, true, true, true],
                vec![true, false, true, false, false, true, false],
                vec![true, false, true, false, false, false, false],
                vec![true, true, false, false, false, false, false],
                vec![false, false, false, false, false, false, false],
            ],
            NoteName::E => vec![
                vec![true, true, true],
                vec![true, false, false],
                vec![true, true, false],
                vec![true, false, false],
                vec![true, true, true]
            ],
            NoteName::F => vec![
                vec![true, true, true],
                vec![true, false, false],
                vec![true, true, true],
                vec![true, false, false],
                vec![true, false, false],
            ],
            NoteName::FSharp => vec![
                vec![false, false, false, false, false, true, false],
                vec![true, true, true, false, true, true, true],
                vec![true, false, false, false, false, true, false],
                vec![true, true, true, false, false, false, false],
                vec![true, false, false, false, false, false, false],
                vec![true, false, false, false, false, false, false],
            ],
            NoteName::G => vec![
                vec![false, true, true, false],
                vec![true, false, false, false],
                vec![true, false, true, true],
                vec![true, false, false, true],
                vec![false, true, true, false]
            ],
            NoteName::GSharp => vec![
                vec![false, false, false, false, false, true, false],
                vec![false, true, true, false, true, true, true],
                vec![true, false, false, false, false, true, false],
                vec![true, false, true, true, false, false, false],
                vec![true, false, false, true, false, false, false],
                vec![false, true, true, false, false, false, false],
            ]
        };

        GraphicalNote {
            matrix,
        }
    }
}

