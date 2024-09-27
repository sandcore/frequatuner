use pitch_detector::core::NoteName;
use crate::EqTunerModeEnum;
use crate::LEDS_MAX_X;
use crate::LEDS_MAX_Y;

/*
Hodgepodge of graphical elements and their rendering. Defined some elements differently than others, depending on their origin. 
This module has conversion functions for the different definitions so the paint function can work with 1 input type.

Could make this more fancy with a better interface but that wouldn't help current demands. If 2d rendering demands increase a point will
be reached where using a full featured public 2D crate is the better choice.
*/

pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl RGB {
    pub fn subtract(&mut self, other_color: RGB) -> [i16; 3] {
        let minuends = [self.r, self.g, self.b];
        let subtrahends = [other_color.r, other_color.g, other_color.b];
        let mut diff_array = [0; 3];

        for (i, minuend) in minuends.iter().enumerate() {
            let diff = *minuend as i16 - subtrahends[i] as i16;
            diff_array[i] = diff;
        }

        diff_array
    }

    pub fn return_new_applied_diff(&mut self, diff: [i16;3]) -> RGB {
        RGB {
            r: (self.r as i16 + diff[0]) as u8,
            g: (self.g as i16 + diff[1]) as u8,
            b: (self.b as i16 + diff[2]) as u8
        }
    }
}

pub fn paint_element(pixelcolors: &mut Vec<u8>, graphic: &Vec<Vec<Option<RGB>>>, x_offset: i32, y_offset: i32) {
    let graphic_width = graphic[0].len();
    let graphic_height = graphic.len();

    if x_offset > LEDS_MAX_X as i32 || y_offset > LEDS_MAX_Y as i32 {
        return //placing the graphic outside of the matrix on the right or top
    }
    if (x_offset + graphic_width as i32) < 0 || (y_offset + graphic_height as i32) < 0 {
        return
    }

    let room_for_drawing_x = LEDS_MAX_X as i32 - x_offset;
    let room_for_drawing_y = LEDS_MAX_Y as i32 - y_offset;

    for (i, row) in graphic.iter().rev().enumerate() { //start rendering at bottom of graphic while all current elements are defined top-down
        for col in 0.. (graphic_width) { //each x
            let mut serpentine = false;
            if (i as i32+y_offset) % 2 == 1 {
                serpentine = true;
            }

            let mut matrix_x = x_offset + col as i32;
            let matrix_y = y_offset + i as i32;

            if (col+1) > room_for_drawing_x as usize || (i+1) > room_for_drawing_y as usize {
                continue
            }
            if matrix_x < 0 || matrix_y < 0 {
                continue
            }

            if serpentine { // serpentine row
                matrix_x = LEDS_MAX_X as i32 - 1 - x_offset - col as i32;
            }

            if let Some(rgb) = &row[col] {
                let index_in_color_vec = (matrix_x*3 + (matrix_y*LEDS_MAX_X as i32)*3) as usize;
                pixelcolors[index_in_color_vec] = rgb.g;
                pixelcolors[index_in_color_vec+1] = rgb.r;
                pixelcolors[index_in_color_vec+2] = rgb.b;
            }
        }
    }
}

pub fn paint_element_rgb(pixelcolors: &mut Vec<RGB>, graphic: &Vec<Vec<Option<RGB>>>, x_offset: i32, y_offset: i32) {
    let graphic_width = graphic[0].len();
    let graphic_height = graphic.len();

    if x_offset > LEDS_MAX_X as i32 || y_offset > LEDS_MAX_Y as i32 {
        return //placing the graphic outside of the matrix on the right or top
    }
    if (x_offset + graphic_width as i32) < 0 || (y_offset + graphic_height as i32) < 0 {
        return
    }

    let room_for_drawing_x = LEDS_MAX_X as i32 - x_offset;
    let room_for_drawing_y = LEDS_MAX_X as i32 - y_offset;

    for (i, row) in graphic.iter().rev().enumerate() { //start rendering at bottom of graphic while all current elements are defined top-down
        for col in 0.. (graphic_width) { //each x
            let mut serpentine = false;
            if (i as i32+y_offset) % 2 == 1 {
                serpentine = true;
            }

            let mut matrix_x = x_offset + col as i32;
            let matrix_y = y_offset + i as i32;

            if (col+1) > room_for_drawing_x as usize || (i+1) > room_for_drawing_y as usize {
                continue
            }
            if matrix_x < 0 || matrix_y < 0 {
                continue
            }

            if serpentine { // serpentine row
                matrix_x = LEDS_MAX_X as i32 - 1 - x_offset - col as i32;
            }

            if let Some(rgb) = &row[col] {
                let index_in_color_vec = (matrix_x + (matrix_y*LEDS_MAX_X as i32)) as usize;
                pixelcolors[index_in_color_vec] = RGB{r:rgb.r, b: rgb.b, g:rgb.g}
            }
        }
    }
}

pub fn convert_vecvecbool_to_xy_rgb_vec(src: Vec<Vec<bool>>, color: RGB) -> Vec<Vec<Option<RGB>>> {
    let rows = src.len();
    let cols = src[0].len();

    let mut dest = vec![];
    for _ in 0.. rows {
        let mut fill_row = vec![];
        for _ in 0.. cols {
            fill_row.push(None);
        }
        dest.push(fill_row);
    }

    for row in 0.. rows {
        for col in 0.. cols {
            if src[row][col] {
                dest[row][col] = Some(RGB{r: color.r, g: color.g, b:color.b});
            }
            else {
                dest[row][col] = None;
            }
        }
    }
    dest
}

// paint_element works on a vec of vecs x*y, convert flattened representation to that
// alpha_num is the RGB value in the bytes that corresponds to a transparent pixel, optionally
fn convert_flatvec_to_xy_rgb_vec( src: Vec<u8>, image_width: usize, image_height: usize, alpha: Option<RGB> ) -> Vec<Vec<Option<RGB>>> {
    let num_colors_width = image_width*3;
    let mut dest_vec = vec![vec![0;num_colors_width]; image_height];

    for i in 0.. image_height {
        for j in 0.. num_colors_width {
            let index_in_src_vec = j + i*num_colors_width;
            dest_vec[i][j] = src[index_in_src_vec];
        }
    }

    let mut result = Vec::new();

    for row in dest_vec {
        let mut row_tuples = Vec::new();
        for chunk in row.chunks_exact(3) {
            if let [r, g, b] = chunk { 
                if let Some(a) = &alpha {
                    if *r==a.r && *g==a.g && *b==a.g {
                        row_tuples.push( None ); // transparent pixel
                    }
                    else {
                        row_tuples.push( Some( RGB {r:*r, g:*g, b:*b}));
                    }
                }
            }
        }
        result.push(row_tuples);
    }

    result
}

/*
Some definitions of graphical elements below here.
*/

pub fn vecvec_one_up() -> Vec<Vec<Option<RGB>>> {
    let one_up = one_up();
    convert_flatvec_to_xy_rgb_vec(one_up, 16, 16, Some(RGB{r:233u8, g:233u8, b:233u8}))
}

pub fn one_up() -> Vec<u8> {
    vec![
        233,233,233,233,233,233,233,233,233,233,233,233,233,233,233,0,0,0,1,0,1,1,0,1,0,0,0,0,0,0,233,233,233,233,233,233,233,233,233,233,233,233,233,233,233,233,233,233,
        233,233,233,233,233,233,233,233,233,233,233,233,0,0,0,0,0,0,255,255,255,83,198,24,82,198,25,255,255,255,0,0,0,233,233,233,233,233,233,233,233,233,233,233,233,233,233,233,
        233,233,233,233,233,233,233,233,233,0,0,0,254,255,254,255,254,255,255,255,254,82,199,24,83,198,24,255,255,255,255,255,255,255,255,255,0,1,0,233,233,233,233,233,233,233,233,233,
        233,233,233,233,233,233,0,0,0,82,198,24,254,255,255,255,255,255,83,199,24,82,199,24,82,198,24,82,198,24,255,255,254,255,255,255,83,198,24,0,0,0,233,233,233,233,233,233,
        233,233,233,1,0,0,255,255,255,82,199,24,82,198,25,83,198,25,82,198,25,82,198,25,82,198,25,83,199,24,82,198,24,82,198,25,83,198,25,254,255,255,0,1,1,233,233,233,
        233,233,233,0,0,0,254,255,255,255,255,255,82,198,25,82,198,24,255,255,255,255,255,255,254,255,254,255,255,255,82,198,25,83,198,24,255,255,255,255,255,254,1,0,0,233,233,233,
        0,0,1,255,255,255,255,255,255,255,255,255,82,198,24,255,255,255,255,255,255,255,255,255,255,255,254,255,254,255,255,255,254,82,198,24,255,254,255,254,254,254,254,254,255,1,0,0,
        0,0,1,255,254,255,255,255,254,255,254,255,82,199,24,255,254,255,255,254,255,254,255,255,255,255,254,255,255,255,255,255,255,82,198,25,255,254,255,255,255,255,255,254,255,0,0,0,
        1,0,1,254,255,255,255,255,255,82,198,24,82,198,24,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,255,82,198,24,82,198,24,254,254,255,254,255,255,0,0,0,
        0,0,1,82,198,24,83,199,24,83,198,24,82,198,24,82,198,24,255,254,255,255,254,254,255,255,254,255,254,255,82,198,24,82,198,24,82,198,24,82,199,24,82,198,25,1,0,0,
        1,0,1,82,198,25,82,198,24,0,0,0,0,0,0,0,0,0,1,0,0,0,1,1,0,0,1,0,0,0,0,1,0,0,0,0,1,0,0,82,198,24,82,199,24,0,0,0,
        233,233,233,0,0,0,0,0,0,0,0,0,254,255,255,254,255,254,1,0,0,255,254,255,255,255,255,1,0,1,254,255,255,255,255,255,0,0,0,1,0,0,0,0,0,233,233,233,
        233,233,233,233,233,233,0,0,0,255,255,255,255,255,255,254,255,255,0,0,0,255,254,255,255,255,255,1,0,0,255,254,255,255,255,254,255,255,255,0,0,0,233,233,233,233,233,233,
        233,233,233,233,233,233,0,1,0,255,255,254,255,255,255,255,255,254,255,255,254,255,255,255,255,255,254,255,254,255,254,254,254,255,254,254,255,254,255,0,0,0,233,233,233,233,233,233,
        233,233,233,233,233,233,233,233,233,1,0,0,255,254,255,255,254,255,255,255,255,254,254,255,255,255,255,254,255,255,255,255,255,255,255,254,1,1,0,233,233,233,233,233,233,233,233,233,
        233,233,233,233,233,233,233,233,233,233,233,233,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,0,0,0,1,0,1,1,0,233,233,233,233,233,233,233,233,233
    ]
}

pub fn line( width: usize, color: RGB ) -> Vec<Vec<Option<RGB>>> {
    let mut row = Vec::with_capacity(width);

    for _ in 0.. width {
        row.push( Some(RGB{r:color.r, g:color.g, b:color.b}));
    }

    vec![row]
}

pub fn dot(color: RGB) -> Vec<Vec<Option<RGB>>> {
    vec![vec![Some(RGB{r:color.r, g:color.g, b:color.b})]]
}

/*
Graphical representations of the musical notes that can be found by NoteDetectionResult
also use the NoteName enum from note_detection crate to have consistency for note names
*/

pub struct GraphicalNote {
    pub matrix: Vec<Vec<bool>>,
}

impl GraphicalNote {
    pub fn new(note: &NoteName) -> Self {
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

pub fn vecvecbool_eq() -> Vec<Vec<bool>> {
    vec![
        vec![true, true, false, true, true, true],
        vec![true, false, false, true, false, true],
        vec![true, true, false, true, true, true],
        vec![true, false, false, false, false, true],
        vec![true, true, false, false, false, true],
    ]
}

pub fn vecvecbool_tuner() -> Vec<Vec<bool>> {
    vec![
        vec![true, true, true, false, false, false],
        vec![false, true, false, false, false, false],
        vec![false, true, false, true, false, true],
        vec![false, false, false, true, false, true],
        vec![false, false, false, true, true, true],
    ]
}

pub fn display_switch_animation(mode: &EqTunerModeEnum) {
    let mut mode_init_screen = Vec::with_capacity(LEDS_MAX_X*LEDS_MAX_Y);
    let mut fill_color_array =  match mode {
        EqTunerModeEnum::Equalizer => vec![1,30,1],
        EqTunerModeEnum::Tuner => vec![1,1,5]
    };

    for _ in 0..(LEDS_MAX_X*LEDS_MAX_Y) {
        mode_init_screen.append(&mut fill_color_array);
    }

    let mut mode_init_animation = mode_init_screen.clone();
    let one_up_graph = vecvec_one_up();
    let eq_graph = convert_vecvecbool_to_xy_rgb_vec(vecvecbool_eq(), RGB{r:0, g:70, b:50});
    let tun_graph = convert_vecvecbool_to_xy_rgb_vec(vecvecbool_tuner(), RGB{r:0, g:70, b:50});

    let mut animation_bg = mode_init_animation.clone();

    match self.mode {
        EqTunerModeEnum::Equalizer => {
            for _ in 0..24 {
                self.switch_element_pos -= 1;
                paint_element(&mut animation_bg, &one_up_graph, self.switch_element_pos, 2);
                paint_element(&mut animation_bg, &eq_graph, 1, 23);

                self.visual_processor.color_vec = animation_bg.clone(); // replace with an initial screen after switch
                self.display_ledmatrix();
                animation_bg = mode_init_animation.clone();
                FreeRtos::delay_ms(100) // bask in the glory of the switch screen
            }
        },
        EqTunerModeEnum::Tuner => {
            for _ in 0..24 {
                self.switch_element_pos += 1;
                paint_element(&mut mode_init_animation, &one_up_graph, self.switch_element_pos, 2);
                paint_element(&mut mode_init_animation, &tun_graph, 1, 23);
                
                self.visual_processor.color_vec = mode_init_animation.clone(); // replace with an initial screen after switch
                self.display_ledmatrix();
                mode_init_animation = mode_init_screen.clone();
                FreeRtos::delay_ms(100) // bask in the glory of the switch screen
            }
        }
    }
    let line_graph = line(LEDS_MAX_X, RGB{r:255, g:216, b:0});
    let dot_graph = dot(RGB{r:40, g: 0, b: 0});
    paint_element(&mut mode_init_screen, &line_graph, 0, 16);
    paint_element(&mut mode_init_screen, &dot_graph, 2, 20);
    paint_element(&mut mode_init_screen, &dot_graph, 4, 20);
    paint_element(&mut mode_init_screen, &dot_graph, 6, 20);

    self.visual_processor.color_vec = mode_init_screen.clone(); // replace with an initial screen after switch
    FreeRtos::delay_ms(200) // bask in the glory of the switch screen
}