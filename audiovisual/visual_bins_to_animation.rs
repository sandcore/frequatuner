use std::f32::consts::PI;

/*
Based on a set of frequency bins with f32 values, an animation is made showing the magnitude of the frequency ranges on the led matrix.

This does NOT take into account when number of frequency bins is not the same as max_y on ledmatrix, it is specifically made for my
aliexpress ledmatrix that takes G R B (for some reason) and serpentines every other row.
*/

struct RGB {
    r: u8,
    g: u8,
    b: u8,
}
impl RGB {
    fn subtract(&mut self, other_color: RGB) -> [i16; 3] {
        let minuends = [self.r, self.g, self.b];
        let subtrahends = [other_color.r, other_color.g, other_color.b];
        let mut diff_array = [0; 3];

        for (i, minuend) in minuends.iter().enumerate() {
            let diff = *minuend as i16 - subtrahends[i] as i16;
            diff_array[i] = diff;
        }

        diff_array
    }

    fn return_new_applied_diff(&mut self, diff: [i16;3]) -> RGB {
        RGB {
            r: (self.r as i16 + diff[0]) as u8,
            g: (self.g as i16 + diff[1]) as u8,
            b: (self.b as i16 + diff[2]) as u8
        }
    }

    fn multiply(&mut self, factor: f32) -> RGB {
        RGB {
            r: (self.r as f32*factor) as u8,
            g: (self.g as f32*factor) as u8,
            b: (self.b as f32*factor) as u8
         }
    }
}


/*
Painter keeps some general state and runs the animation process.
*/
pub struct Painter {
    led_matrix_max_x: usize,
    led_matrix_max_y: usize,

    // some paint state that is needed between iterations
    iteration: u16,
    background_cycle_state: f32,
    bar_cycle_state: f32,
    current_bg_color: RGB,
    bar_ghosts: Vec<Option<RGB>>, // for fadeout of previous bars
}
impl Painter {
    pub fn new(x: usize, y: usize) -> Self {
        let mut bar_ghosts = Vec::with_capacity(x*y);
        for i in 0..x*y {
            bar_ghosts.push(None);
        }

        Painter {
            led_matrix_max_x: x,
            led_matrix_max_y: y,

            // some paint state that is needed between iterations
            iteration: 0,
            background_cycle_state: 0.0,
            bar_cycle_state: 0.0,
            current_bg_color: RGB{r:0, g:0, b:0},
            bar_ghosts
        }
    }

    pub fn paint(&mut self, eq_bins: &Vec<f32>) -> Vec<u8> { // go from a blank canvas to a painted canvas
        let blank_canvas = BlankCanvas::new(self.led_matrix_max_x, self.led_matrix_max_y);        
        let background_drawn = blank_canvas.draw_background(self);
        let faded_bars_drawn = background_drawn.draw_fade_bars(self);
        let new_bars_drawn = faded_bars_drawn.draw_new_bars(self, eq_bins);

        new_bars_drawn.output()
    }
}

/*
Typestate pattern:
 - Initial blank canvas
 - Draw background
 - Draw fade bars (ghost of previously displayed bar)
 - Draw new bars
 After each state transition the previous state is destroyed

 Did not want to use a trait object for the shared / state stuff because I wanted to minimize run-time impact on embedded.
*/

struct BlankCanvas {
    color_vec: Vec<RGB>,
    
    // background color variables
    iterations_between_bg_refreshes: u16,
    background_min_value: u8,
    background_max_value: u8,
}
struct BackgroundDrawn {
    color_vec: Vec<RGB>,

    // fade settings
    fade_factor: f32
}
struct FadedBarsDrawn {
    color_vec: Vec<RGB>,

    // newbar color settings
    newbar_color: RGB,
    newbar_min_intensity: u8,
    newbar_max_intensity: u8,
}
struct NewBarsDrawn {
    color_vec: Vec<RGB>
}

impl BlankCanvas {
    fn new(max_x: usize, max_y: usize) -> BlankCanvas {
        let mut empty_canvas = vec![];
        for i in 0..(max_x*max_y) {
            empty_canvas.push(RGB{r:1,g:1,b:1});
        }

        BlankCanvas {
            color_vec: empty_canvas,
            iterations_between_bg_refreshes: 5,
            background_min_value: 1,
            background_max_value: 5
        }
    }

    // background is a pulsing animation
    fn draw_background(mut self, painter: &mut Painter) -> BackgroundDrawn {
        painter.iteration += 1;

        if painter.iteration % self.iterations_between_bg_refreshes == 0 {
            let cycle_state = self.get_new_cycle_state(painter.background_cycle_state);
            painter.background_cycle_state = cycle_state;
            painter.iteration = 0;
        }
      
        painter.current_bg_color = self.get_bg_based_on_cycle_state(&painter.background_cycle_state);
        
        for i in 0 .. painter.led_matrix_max_x as usize * painter.led_matrix_max_y as usize {
            self.color_vec[i] = RGB{r:painter.current_bg_color.r, g:painter.current_bg_color.g, b:painter.current_bg_color.b};
        }
        
        BackgroundDrawn::new(self.color_vec)
    }

    fn get_new_cycle_state(&mut self, mut cycle_state: f32) -> f32 {
        cycle_state += 0.06;
        if cycle_state >= 2.0 {
            cycle_state -= 2.0;
        }
        cycle_state
    }

    fn get_bg_based_on_cycle_state(&mut self, cycle_state: &f32) -> RGB {
        let range = (self.background_max_value - self.background_min_value) as f32;
        let green = (self.background_min_value as f32 + range * (0.8 + 0.2 * (cycle_state * PI).sin())).round() as u8;
        let red = (self.background_min_value as f32 + range * (0.5 - 0.5 * ((cycle_state + 0.5) * PI).cos())).round() as u8;
        let blue = self.background_min_value;
    
        RGB{r: red, g: green, b: blue}
    }
}

impl BackgroundDrawn {
    fn new(color_vec: Vec<RGB>) -> Self {
        BackgroundDrawn {
            color_vec,
            fade_factor: 0.6
        }
    }
    fn draw_fade_bars(mut self, painter: &mut Painter) -> FadedBarsDrawn {
        // For every barghosts entry that is fading, fade it some more. 
        //  - If it's near the background color, stop fading
        //  - Otherwise Update the color_vec and barghosts with the more faded color entry
        for i in 0..self.color_vec.len() {
            if let Some(current_ghost_bar_color) = &mut painter.bar_ghosts[i] {
                let mut current_color = RGB {r: current_ghost_bar_color.r, g: current_ghost_bar_color.g, b:current_ghost_bar_color.b};
                let mut desired_color = RGB {r: painter.current_bg_color.r, g: painter.current_bg_color.g, b:painter.current_bg_color.b};

                let diff_colors = desired_color.subtract(current_color); // the full change that is needed to reach the desired color
                let threshold = 4;
                
                if diff_colors[0].abs() <= threshold && diff_colors[1].abs() <= threshold && diff_colors[2].abs() <= threshold {
                    painter.bar_ghosts[i] = None; // don't need to fade this anymore, value within threshold
                }
                else {
                    let diff_with_fade_factor = std::array::from_fn(|i| {
                        (diff_colors[i] as f32 * self.fade_factor).round() as i16 // the current step towards desired we're taking this canvas paint run
                    });
                    let stepped_color = current_ghost_bar_color.return_new_applied_diff(diff_with_fade_factor);
    
                    self.color_vec[i] = RGB{r:stepped_color.r, g:stepped_color.g, b:stepped_color.b};
                    painter.bar_ghosts[i] = Some(RGB{r:stepped_color.r, g:stepped_color.g, b:stepped_color.b});
                }
            }
        }

        FadedBarsDrawn::new(self.color_vec)
    }
}

impl FadedBarsDrawn {
    fn new(color_vec: Vec<RGB>) -> Self {
        FadedBarsDrawn {
            newbar_color: RGB{ r:0, g:0, b:60 },
            newbar_min_intensity: 4,
            newbar_max_intensity: 60,
            color_vec
        }
    }
    fn draw_new_bars(mut self, painter: &mut Painter, eq_bins: &Vec<f32>) -> NewBarsDrawn {
        // equalizer magnitudes displayed as rows on a portrait ledmatrix. Every bin corresponds 1:1 to a led matrix Y.
        for row in 0.. eq_bins.len() {
            let mut serpentine = false;

            //magnitude of frequency bin expressed in number of leds
            let amount_leds_mag = (painter.led_matrix_max_x as f32 * eq_bins[row]).ceil().clamp(0.0, painter.led_matrix_max_x as f32) as u32;

            if row % 2 == 0 { // on a serpentine row, the ledmatrix renders from last led to the first 
                serpentine = true;
            }
            self.draw_row(painter, row as u32, amount_leds_mag, serpentine, painter.led_matrix_max_x as u32, &eq_bins[row]);
        }

        NewBarsDrawn {
            color_vec: self.color_vec
        }
    }

    fn draw_row(&mut self, painter: &mut Painter, row_num: u32, magnitude: u32, serpentine: bool, row_size: u32, bin_amplitude: &f32) {
        if magnitude == 0 {
            return // there is no bar to draw
        }
        let row_indexes_size = row_size-1;
        let magni_as_index = magnitude - 1;
        let first_led_of_row_num_index = row_num * row_size;
        let color_vec_start_index;
        let color_vec_end_index;

        match serpentine { // inverse magnitude drawing on serpentine rows
            false => {
                color_vec_start_index = first_led_of_row_num_index;
                color_vec_end_index = first_led_of_row_num_index + magni_as_index;
            },
            true => {
                color_vec_start_index = first_led_of_row_num_index + row_indexes_size - magni_as_index;
                color_vec_end_index = first_led_of_row_num_index + row_indexes_size;
            }
        }

        let newbar_color = self.get_newbar_color(painter, bin_amplitude);

        for i in color_vec_start_index as usize..=color_vec_end_index as usize {
            painter.bar_ghosts[i] = Some(RGB{r:newbar_color.r, g:newbar_color.g, b:newbar_color.b}); // next loop start fading these new bar leds again from full
            self.color_vec[i] = RGB{r:newbar_color.r, g:newbar_color.g, b:newbar_color.b};
        }
    }

    fn get_newbar_color(&mut self, painter: &mut Painter, bin_amplitude: &f32) -> RGB {
        let bar_fill_intensity = (self.newbar_max_intensity as f32 * bin_amplitude).ceil().clamp(self.newbar_min_intensity as f32,self.newbar_max_intensity as f32) as u8;

        // cycle bar colour
        painter.bar_cycle_state += 0.001;
        if painter.bar_cycle_state > 2.0 * PI {
            painter.bar_cycle_state -= 2.0 * PI;
        }

        let blue_factor = (painter.bar_cycle_state.sin() + 1.0) / 2.0;
        let red_factor = (1.0 - blue_factor) * 0.1;
        
        // Ensure the total intensity stays the same
        let total_factor = blue_factor + red_factor;
        let intensity_scale = bar_fill_intensity as f32 / total_factor;
        
        let blue = (blue_factor * intensity_scale).round() as u8;
        let red = (red_factor * intensity_scale).round() as u8;

        RGB {
            r: red,
            g: 0,
            b: blue
        }
    }
}

impl NewBarsDrawn {
    fn output(self) -> Vec<u8> {
        // led matrix needs a vec of GRB values
        self.color_vec.into_iter().flat_map(|rgb| [rgb.g, rgb.r, rgb.b]).collect()
    }
}