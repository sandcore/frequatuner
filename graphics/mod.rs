pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

pub fn paint_element(pixelcolors: &mut Vec<u8>, graphic: &Vec<Vec<Option<RGB>>>, x_offset: i32, y_offset: i32, max_x: usize, max_y: usize) {
    let graphic_width = graphic[0].len();
    let graphic_height = graphic.len();

    if x_offset > max_x as i32 || y_offset > max_y as i32 {
        return //placing the graphic outside of the matrix on the right or top
    }
    if (x_offset + graphic_width as i32) < 0 || (y_offset + graphic_height as i32) < 0 {
        return
    }

    let room_for_drawing_x = max_x as i32 - x_offset;
    let room_for_drawing_y = max_y as i32 - y_offset;

    for (i, row) in graphic.iter().rev().enumerate() { //start rendering at bottom of graphic
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
                matrix_x = max_x as i32 - 1 - x_offset - col as i32;
            }

            if let Some(rgb) = &row[col] {
                let index_in_color_vec = (matrix_x*3 + (matrix_y*max_x as i32)*3) as usize;
                pixelcolors[index_in_color_vec] = rgb.g;
                pixelcolors[index_in_color_vec+1] = rgb.r;
                pixelcolors[index_in_color_vec+2] = rgb.b;
            }
        }
    }
}


pub fn c() -> Vec<Vec<Option<u8>>> {
vec![
        vec![Some(1),Some(1), Some(1)],
        vec![Some(1), Some(1), Some(1)],
        vec![Some(1), Some(1), Some(1)],
        vec![Some(1), Some(1), Some(1)],
    ]
}

pub fn vecvec_one_up() -> Vec<Vec<Option<RGB>>> {
    let one_up = one_up();
    convert_flatvec_to_xyvec(one_up, 48, 16, Some(RGB{r:233u8, g:233u8, b:233u8}))
}

// paint_element works on a vec of vecs x*y, convert flattened representation to that
// alpha_num is the RGB value in the bytes that corresponds to a transparent pixel, optionally
fn convert_flatvec_to_xyvec( src: Vec<u8>, image_width: usize, image_height: usize, alpha: Option<RGB> ) -> Vec<Vec<Option<RGB>>> {
    let mut dest_vec = vec![vec![0;image_width]; image_height];

    for i in 0.. image_height {
        for j in 0.. image_width {
            let index_in_src_vec = j + i*image_width;
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