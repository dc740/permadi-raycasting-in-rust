use crate::loader::Assets;
use minifb::{Key, Window};
use std::collections::HashMap;
use std::f64::consts::PI;
/**********************************************
Raycasting implementation in Rust.
Original port: https://github.com/permadi-com/ray-cast/tree/master/demo/7

What's on this demo:
Wall finding
Generating lookup tables
Fishbowl / distortion corrections
Rendering of simple (static) ground and sky
Movement handling
Textured wall
Collision detection
Double buffering
Floor casting
Ceiling Casting
Vertical motions technique (by altering player's height and projection plane)
---------------

License: MIT (https://opensource.org/licenses/MIT)

Copyright 2022 Emilio Moretti

Permission is hereby granted, free of charge, to any person obtaining a copy of this
software and associated documentation files (the "Software"),
to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to permit persons to
whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

***********************************************/

#[inline]
pub fn clamp_i32_to_u8(value: i32) -> u8 {
    let mut x = value;
    if x < 0 {
        x = 0;
    }
    if x > 255 {
        x = 255;
    }
    x as u8
}

#[inline]
pub fn clamp_u16_to_u8(value: u16) -> u8 {
    (value & 0xffff) as u8
}

#[inline]
pub fn u8_to_color(alpha: u8, red: u8, green: u8, blue: u8) -> u32 {
    #[cfg(not(feature = "web"))]
    {
        ((alpha as u32) << 24) | ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
    }
    #[cfg(feature = "web")]
    {
        ((alpha as u32) << 24) | ((blue as u32) << 16) | ((green as u32) << 8) | (red as u32)
    }
}

#[inline]
pub fn clamp_u32_to_u8(value: u32) -> u8 {
    let mut x = value;
    if x > 255 {
        x = 255;
    }
    x as u8
}

pub struct GameWindow {
    width: u32,
    height: u32,
    //framerate: u32,
    buffer_n: usize,
    area_size: usize,
    canvas: Vec<u32>,
    pub assets: Assets,
    // size of tile (wall height)
    tile_size: u32,
    wall_height: u32,

    // Remember that PROJECTIONPLANE = screen.  This demo assumes your screen is 320 pixels wide, 200 pixels high
    projectionplanewidth: u32,
    projectionplaneheight: u32,

    // We use FOV of 60 degrees.  So we use this FOV basis of the table, taking into account
    // that we need to cast 320 rays (PROJECTIONPLANEWIDTH) within that 60 degree FOV.
    //angle60: u32,
    // You must make sure these values are integers because we're using loopup tables.
    angle30: u32,
    //angle15: u32,
    angle90: u32,
    angle180: u32,
    angle270: u32,
    angle360: u32,
    angle0: u32,
    angle5: u32,
    //angle3: u32,
    //angle10: u32,
    //angle45: u32,

    // trigonometric tables (the ones with "I" such as ISiTable are "Inverse" table)
    f_sin_table: Vec<f32>,
    f_isin_table: Vec<f32>,
    f_cos_table: Vec<f32>,
    f_icos_table: Vec<f32>,
    f_tan_table: Vec<f32>,
    f_itan_table: Vec<f32>,
    f_fish_table: Vec<f32>,
    f_xstep_table: Vec<f32>,
    f_ystep_table: Vec<f32>,

    // player's attributes
    f_player_x: f32,
    f_player_y: f32,
    f_player_arc: i32,
    f_player_distance_to_the_projection_plane: f32,
    f_player_height: i32,
    f_player_speed: u32,

    // Half of the screen height
    f_projection_plane_ycenter: i32,

    // the following variables are used to keep the player coordinate in the overhead map
    f_player_map_x: f32,
    f_player_map_y: f32,
    f_minimap_width: u32,

    // movement flag
    f_key_up: bool,
    f_key_down: bool,
    f_key_left: bool,
    f_key_right: bool,
    f_key_look_up: bool,
    f_key_look_down: bool,
    f_key_fly_up: bool,
    f_key_fly_down: bool,
    f_key_ceiling_toggle: bool,
    no_ceiling: bool,

    // 2 dimensional map
    f_map: String,
    map_width: u32,
    map_height: u32,

    f_background_image_arc: i32,

    base_light_value: i32,
}

impl GameWindow {
    pub fn new(width: usize, height: usize) -> Self {
        let buffer_len: usize = (width * height) * 4 * 2;
        let canvas: Vec<u32> = vec![0; buffer_len];
        let projectionplanewidth = 320;
        let projectionplaneheight = 200;
        let angle60 = projectionplanewidth;
        // You must make sure these values are integers because we're using loopup tables.
        let angle30 = angle60 / 2;
        //let angle15 = angle30 / 2;
        let angle90 = angle30 * 3;
        let angle180 = angle90 * 2;
        let angle270 = angle90 * 3;
        let angle360 = angle60 * 6;
        let angle0 = 0;
        let angle5 = angle30 / 6;
        //let angle3 = angle30 / 10;
        //let angle10 = angle5 * 2;
        //let angle45 = angle15 * 3;
        let gw = GameWindow {
            width: width as u32,
            height: height as u32,
            //framerate: 24,
            buffer_n: 0,
            area_size: (width * height),
            // create the main canvas
            canvas,
            assets: Assets {
                root: "http://localhost".to_string(), //FIXME: make this better
                textures: HashMap::new(),
            },
            // size of tile (wall height)
            tile_size: 64,
            wall_height: 64,

            // Remember that PROJECTIONPLANE = screen.  This demo assumes your screen is 320 pixels wide, 200 pixels high
            projectionplanewidth,
            projectionplaneheight,

            // We use FOV of 60 degrees.  So we use this FOV basis of the table, taking into account
            // that we need to cast 320 rays (PROJECTIONPLANEWIDTH) within that 60 degree FOV.
            //angle60,
            // You must make sure these values are integers because we're using loopup tables.
            angle30,
            //angle15,
            angle90,
            angle180,
            angle270,
            angle360,
            angle0,
            angle5,
            //angle3,
            //angle10,
            //angle45,

            // trigonometric tables (the ones with "I" such as ISiTable are "Inverse" table)
            f_sin_table: vec![0.0; angle360 as usize + 1],
            f_isin_table: vec![0.0; angle360 as usize + 1],
            f_cos_table: vec![0.0; angle360 as usize + 1],
            f_icos_table: vec![0.0; angle360 as usize + 1],
            f_tan_table: vec![0.0; angle360 as usize + 1],
            f_itan_table: vec![0.0; angle360 as usize + 1],
            f_fish_table: vec![0.0; angle360 as usize + 1],
            f_xstep_table: vec![0.0; angle360 as usize + 1],
            f_ystep_table: vec![0.0; angle360 as usize + 1],

            // player's attributes
            f_player_x: 100.0,
            f_player_y: 160.0,
            f_player_arc: angle60 as i32,
            f_player_distance_to_the_projection_plane: 277.0,
            f_player_height: 32,
            f_player_speed: 16,

            // Half of the screen height
            f_projection_plane_ycenter: projectionplaneheight as i32 / 2,

            // the following variables are used to keep the player coordinate in the overhead map
            f_player_map_x: 0.0,
            f_player_map_y: 0.0,
            f_minimap_width: 5,

            // movement flag
            f_key_up: false,
            f_key_down: false,
            f_key_left: false,
            f_key_right: false,
            f_key_look_up: false,
            f_key_look_down: false,
            f_key_fly_up: false,
            f_key_fly_down: false,
            f_key_ceiling_toggle: false,
            no_ceiling: false,

            // 2 dimensional map
            f_map: "".to_string(),
            map_width: 20,
            map_height: 20,
            //            animation_frame_id: 0,

            //fWallTextureCanvas,
            //fWallTexturePixels,
            f_background_image_arc: 0,

            base_light_value: 180,
            //base_light_value_delta: 1,
        };
        return gw;
    }

    //*******************************************************************//
    //* Convert arc (degree) to radian
    //*******************************************************************//
    fn arc_to_rad(&self, arc_angle: f32) -> f32 {
        return (arc_angle * PI as f32) / self.angle180 as f32;
    }

    //*******************************************************************//
    //* Mostly used to draw in the overhead map. Doesn't have other uses now.
    //*******************************************************************//
    fn draw_line(
        &mut self,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    ) {
        let default_increment: i32 = 1; // we access the canvas 4 bytes at a time
        let x_increment: i32;
        let y_increment: i32;

        let canvas_len = self.canvas.len() as i32;
        // calculate Ydistance
        let mut dy: i32 = end_y - start_y;

        // if moving negative dir (up)
        // note that we can simplify this function if we can guarantee that
        // the line will always move in one direction only
        if dy < 0 {
            // get abs
            dy = -dy;
            // negative movement
            y_increment = -(self.width as i32) * default_increment;
        } else {
            y_increment = self.width as i32 * default_increment;
        }
        // calc x distance
        let mut dx: i32 = end_x - start_x;

        // if negative dir (left)
        // note that we can simplify this function if we can guarantee that
        // the line will always move in one direction only
        if dx < 0 {
            dx = -dx;
            x_increment = -(default_increment);
        } else {
            x_increment = default_increment;
        }
        // deflation
        let mut error = 0;
        let mut target_index: i32 =
            default_increment * self.width as i32 * start_y + default_increment * start_x;

        // if movement in x direction is larger than in y
        // ie: width > height
        // we draw each row one by one
        if dx > dy {
            // length = width +1
            let length = dx;

            for _i in 0..length {
                if target_index < 0 {
                    break;
                }
                if target_index < canvas_len {
                    //ARGB
                    self.canvas[target_index as usize] = u8_to_color(alpha, red, green, blue);
                }

                // either move left/right
                target_index = target_index + x_increment;
                // cumulate error term
                error += dy;

                // is it time to move y direction (chage row)
                if error >= dx {
                    error -= dx;
                    // move to next row
                    target_index = target_index + y_increment as i32;
                }
            }
        }
        // if movement in y direction is larger than in x
        // ie: height > width
        // we draw each column one by one
        // note that a diagonal line will go here because xdiff = ydiff
        else
        //(YDiff>=XDiff)
        {
            let length = dy;

            for _i in 0..length {
                if target_index < 0 {
                    break;
                }

                if target_index < canvas_len {
                    //ARGB
                    self.canvas[target_index as usize] = u8_to_color(alpha, red, green, blue);
                }
                target_index = target_index + y_increment as i32;
                error += dx;

                if error >= dy {
                    error -= dy;
                    target_index = target_index + x_increment as i32;
                }
            }
        }
    }

    fn draw_wall_slice_rectangle_tinted(
        &mut self,
        x_param: f32,
        y_param: f32,
        _width: f32,
        height: f32,
        x_offset_param: f32,
        brightness_level: f32,
    ) {
        // wait until the texture loads
        if !self.assets.textures.contains_key("/images/tile2.ff") {
            return;
        }
        let f_wall_texture_buffer;
        match self.assets.textures.get("/images/tile2.ff") {
            None => panic!("Wall not loaded. Dunno what to do"),
            Some(wall_texture) => f_wall_texture_buffer = wall_texture,
        }
        let _dy = height;
        let x = x_param.floor();
        let y = y_param.floor();
        let x_offset = x_offset_param.floor();
        let bytes_per_pixel: u32 = 4;

        let mut source_index = bytes_per_pixel * x_offset as u32;

        let last_source_index =
            f_wall_texture_buffer.width * f_wall_texture_buffer.height * bytes_per_pixel - 4;

        //let targetCanvasPixels=self.canvasContext.createImageData(0, 0, width, height);
        let mut target_index: i32 = ((self.width * 1) as f32 * y + (1 as f32 * x)) as i32;
        let canvas_len: usize = self.canvas.len();
        let mut height_to_draw: f32 = height;
        // clip bottom
        if y + height_to_draw > self.height as f32 {
            height_to_draw = self.height as f32 - y;
        }

        let mut y_error: f32 = 0.0;

        // we need to check this, otherwise, program might crash when trying
        // to fetch the shade if this condition is true (possible if height is 0)
        if height_to_draw < 0.0 {
            return;
        }

        // we're going to draw the first row, then move down and draw the next row
        // and so on we can use the original x destination to find out
        // the x position of the next row
        // Remeber that the source bitmap is rotated, so the width is actually the
        // height
        loop {
            // if error < actualHeight, this will cause row to be skipped until
            // this addition sums to scaledHeight
            // if error > actualHeight, this ill cause row to be drawn repeatedly until
            // this addition becomes smaller than actualHeight
            // 1) Think the image height as 100, if percent is >= 100, we'll need to
            // copy the same pixel over and over while decrementing the percentage.
            // 2) Similarly, if percent is <100, we skip a pixel while incrementing
            // and do 1) when the percentage we're adding has reached >=100
            y_error += height;

            // dereference for faster access (especially useful when the same bit
            // will be copied more than once)

            // Cheap shading trick by using brightnessLevel (which doesn't really have to correspond to "brightness")
            // to alter colors.  You can use logarithmic falloff or linear falloff to produce some interesting effect
            let f_wall_texture_pixels = &f_wall_texture_buffer.data;
            let red = clamp_u16_to_u8(f_wall_texture_pixels[source_index as usize]) as f32
                * brightness_level; //.floor();
            let green = clamp_u16_to_u8(f_wall_texture_pixels[source_index as usize + 1]) as f32
                * brightness_level; //.floor();
            let blue = clamp_u16_to_u8(f_wall_texture_pixels[source_index as usize + 2]) as f32
                * brightness_level; //.floor();
            let alpha = clamp_u16_to_u8(f_wall_texture_pixels[source_index as usize + 3]); //.floor();

            // while there's a row to draw & not end of drawing area
            while y_error >= f_wall_texture_buffer.width as f32 {
                y_error -= f_wall_texture_buffer.width as f32;
                if (target_index as usize) < canvas_len {
                    self.canvas[target_index as usize] = u8_to_color(
                        alpha,
                        red.floor() as u8,
                        green.floor() as u8,
                        blue.floor() as u8,
                    ); //0xFF005500;/**/
                }
                target_index += (1 * self.width) as i32;

                // clip bottom (just return if we reach bottom)
                height_to_draw -= 1.0;
                if height_to_draw < 1.0 {
                    return;
                }
            }
            source_index += bytes_per_pixel * f_wall_texture_buffer.width;
            if source_index > last_source_index {
                source_index = last_source_index;
            }
        }
    }

    fn clear_offscreen_canvas(&self) {
        // no need to do anything because the screen will be redrwan fully anyway
    }
    //*******************************************************************//
    //* Mostly used to draw in the overhead map. Doesn't have other uses now.
    //*******************************************************************//
    fn draw_fill_rectangle(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    ) {
        let canvas_len: usize = self.canvas.len();
        let default_increment = 1;
        //let targetCanvasPixels=self.canvasContext.createImageData(0, 0, width, height);
        let mut target_index: i32 =
            (default_increment * self.width * y + default_increment * x) as i32;
        for _h in 0..height {
            for _w in 0..width {
                //ARGB
                if (target_index as usize) < canvas_len {
                    self.canvas[target_index as usize] = u8_to_color(alpha, red, green, blue);
                }
                target_index += default_increment as i32;
            }
            target_index += (default_increment * (self.width - width)) as i32;
        }
    }

    pub fn init(&mut self) {
        let mut radian;
        self.f_sin_table = vec![0.0; self.angle360 as usize + 1];
        self.f_isin_table = vec![0.0; self.angle360 as usize + 1];
        self.f_cos_table = vec![0.0; self.angle360 as usize + 1];
        self.f_icos_table = vec![0.0; self.angle360 as usize + 1];
        self.f_tan_table = vec![0.0; self.angle360 as usize + 1];
        self.f_itan_table = vec![0.0; self.angle360 as usize + 1];
        self.f_fish_table = vec![0.0; self.angle360 as usize + 1];
        self.f_xstep_table = vec![0.0; self.angle360 as usize + 1];
        self.f_ystep_table = vec![0.0; self.angle360 as usize + 1];

        for i in 0..=self.angle360 as usize {
            // Populate tables with their radian values.
            // (The addition of 0.0001 is a kludge to avoid divisions by 0. Removing it will produce unwanted holes in the wall when a ray is at 0, 90, 180, or 270 degree angles)
            radian = self.arc_to_rad(i as f32) + 0.0001;
            self.f_sin_table[i] = radian.sin();
            self.f_isin_table[i] = 1.0 / self.f_sin_table[i];
            self.f_cos_table[i] = radian.cos();
            self.f_icos_table[i] = 1.0 / self.f_cos_table[i];
            self.f_tan_table[i] = radian.tan();
            self.f_itan_table[i] = 1.0 / self.f_tan_table[i];

            // Next we crate a table to speed up wall lookups.
            //
            //  You can see that the distance between walls are the same
            //  if we know the angle
            //  _____|_/next xi______________
            //       |
            //  ____/|next xi_________   slope = tan = height / dist between xi's
            //     / |
            //  __/__|_________  dist between xi = height/tan where height=tile size
            // old xi|
            //                  distance between xi = x_step[view_angle];

            // Facing LEFT
            if i >= (self.angle90 as usize) && i < (self.angle270 as usize) {
                self.f_xstep_table[i] = self.tile_size as f32 / self.f_tan_table[i];
                if self.f_xstep_table[i] > 0.0 {
                    self.f_xstep_table[i] = -self.f_xstep_table[i];
                }
            }
            // facing RIGHT
            else {
                self.f_xstep_table[i] = self.tile_size as f32 / self.f_tan_table[i];
                if self.f_xstep_table[i] < 0.0 {
                    self.f_xstep_table[i] = -self.f_xstep_table[i];
                }
            }

            // FACING DOWN
            if i >= (self.angle0 as usize) && (i as u32) < self.angle180 {
                self.f_ystep_table[i] = self.tile_size as f32 * self.f_tan_table[i];
                if self.f_ystep_table[i] < 0.0 {
                    self.f_ystep_table[i] = -self.f_ystep_table[i];
                }
            }
            // FACING UP
            else {
                self.f_ystep_table[i] = self.tile_size as f32 * self.f_tan_table[i];
                if self.f_ystep_table[i] > 0.0 {
                    self.f_ystep_table[i] = -self.f_ystep_table[i];
                }
            }
        }

        // Create table for fixing FISHBOWL distortion
        for i in -(self.angle30 as i32)..=self.angle30 as i32 {
            radian = self.arc_to_rad(i as f32);
            // we don't have negative angle, so make it start at 0
            // this will give range from column 0 to 319 (PROJECTONPLANEWIDTH) since we only will need to use those range
            self.f_fish_table[(i + self.angle30 as i32) as usize] = 1.0 / radian.cos();
        }

        // CREATE A SIMPLE MAP.
        // Use string for elegance (easier to see).  W=Wall, O=Opening

        let map3 = "WWWWWWWWWWWWWWWWWWWW\
                WOOOOOOOOOOOOOOOOOOW\
                WOOWOWOWOOOWOOWWOWOW\
                WOOOOOOOWOOWOOOOWWOW\
                WOOWOWOOWOOWOOWWWWOW\
                WOOWOWWOWOOWOOWOOWOW\
                WOOWOOWOWOOWOOWOOWOW\
                WOOOWOWOWOOWOOOOOWOW\
                WOOOWOWOWOOWOOWOOWOW\
                WOOOWWWOWOOWOOWWWWOW\
                WOOOOOOOOOOOOOOOOOOW\
                WOOWOWOWOOOWOOWWOWOW\
                WOOOOOOOWOOWOOOOOWOW\
                WOOWOWOWOOOWOOWWOWOW\
                WOOOOOOOWOOWOOOOOWOW\
                WOOWOWOWOOOWOOWWOWOW\
                WOOOOOOOWOOOOOOOOWOW\
                WOOWOWOWOOOWOOWWOWOW\
                WOOOOOOOOOOWOOOOOOOW\
                WWWWWWWWWWWWWWWWWWWW";
        self.f_map = map3.to_string();
        self.map_width = 20;
        self.map_height = 20;
    }

    //*******************************************************************//
    //* Draw map on top. Draw a black squares.
    //*******************************************************************//
    fn draw_overhead_map(&mut self) {
        for r in 0..self.map_height {
            for c in 0..self.map_width {
                if self
                    .f_map
                    .chars()
                    .nth((r * self.map_width + c) as usize)
                    .unwrap()
                    != 'O'
                {
                    self.draw_fill_rectangle(
                        c * self.f_minimap_width, //self.projectionplanewidth + (c * self.f_minimap_width),
                        r * self.f_minimap_width,
                        self.f_minimap_width,
                        self.f_minimap_width,
                        0,
                        0,
                        0,
                        255,
                    );
                }
            }
        }
        // Draw player position on the overhead map
        self.f_player_map_x =
            (self.f_player_x / self.tile_size as f32) * self.f_minimap_width as f32; //self.projectionplanewidth as f32 + ((self.f_player_x / self.tile_size as f32) * self.f_minimap_width as f32);
        self.f_player_map_y =
            (self.f_player_y / self.tile_size as f32) * self.f_minimap_width as f32;
    }

    //*******************************************************************//
    //* Draw background image
    //*******************************************************************//

    fn draw_background(&mut self) {
        let width = self.projectionplanewidth as i32 - self.f_background_image_arc;
        let height = self.projectionplaneheight as i32;
        let src_width = self.assets.textures["/images/bgr.ff"].width;
        let bytes_per_pixel = 4;
        let canvas_len = self.canvas.len();
        for source_x in 0..width {
            let mut dest_x = self.f_background_image_arc + source_x;

            if dest_x > width {
                dest_x -= width;
            } else if dest_x < 0 {
                dest_x = self.projectionplanewidth as i32 + dest_x
            }
            for source_y in 0..height {
                let dest_y = source_y;

                let target_index_src = (source_y * src_width as i32 + source_x) * bytes_per_pixel;
                let target_index_dest = dest_y * self.width as i32 + dest_x;

                let red = clamp_u16_to_u8(
                    self.assets.textures["/images/bgr.ff"].data[target_index_src as usize],
                ) as f32;
                let green = clamp_u16_to_u8(
                    self.assets.textures["/images/bgr.ff"].data[target_index_src as usize + 1],
                ) as f32;
                let blue = clamp_u16_to_u8(
                    self.assets.textures["/images/bgr.ff"].data[target_index_src as usize + 2],
                ) as f32;

                let alpha = clamp_u16_to_u8(
                    self.assets.textures["/images/bgr.ff"].data[target_index_src as usize + 3],
                );

                if (target_index_dest as usize) < canvas_len {
                    // Draw the pixel
                    self.canvas[target_index_dest as usize] = u8_to_color(
                        alpha,
                        red.floor() as u8,
                        green.floor() as u8,
                        blue.floor() as u8,
                    );
                }
            }
        }
    }

    //*******************************************************************//
    //* Draw ray on the overhead map (for illustartion purpose)
    //* This is not part of the ray-casting process
    //*******************************************************************//

    fn draw_ray_on_overhead_map(
        &mut self,
        x: f32,
        y: f32,
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    ) {
        // draw line from the player position to the position where the ray
        // intersect with wall
        self.draw_line(
            self.f_player_map_x.floor() as i32,
            self.f_player_map_y.floor() as i32,
            //(self.projectionplanewidth as f32 + (x * self.f_minimap_width as f32) / self.tile_size as f32).floor() as i32,
            ((x * self.f_minimap_width as f32) / self.tile_size as f32).floor() as i32,
            ((y * self.f_minimap_width as f32) / self.tile_size as f32).floor() as i32,
            red,
            green,
            blue,
            alpha,
        );
    }

    //*******************************************************************//
    //* Draw player POV on the overhead map (for illustartion purpose)
    //* This is not part of the ray-casting process
    //*******************************************************************//
    fn draw_player_pov_on_overhead_map(&mut self, _x: u32, _y: u32) {
        // draw a red line indication the player's direction
        self.draw_line(
            self.f_player_map_x.floor() as i32,
            self.f_player_map_y.floor() as i32,
            (self.f_player_map_x as f32 + self.f_cos_table[self.f_player_arc as usize] * 10.0)
                .floor() as i32,
            (self.f_player_map_y as f32 + self.f_sin_table[self.f_player_arc as usize] * 10.0)
                .floor() as i32,
            255,
            0,
            0,
            255,
        );
    }

    //*******************************************************************//
    //* Renderer
    //*******************************************************************//
    fn raycast(&mut self) {
        let mut vertical_grid; // horizotal or vertical coordinate of intersection
        let mut horizontal_grid: f32; // theoritically, this will be multiple of TILE_SIZE
                                      // , but some trick did here might cause
                                      // the values off by 1
        let mut dist_to_next_vertical_grid: f32; // how far to the next bound (this is multiple of
        let mut dist_to_next_horizontal_grid: f32; // tile size)
        let mut x_intersection: f32; // x and y intersections
        let mut y_intersection: f32;
        let mut dist_to_next_xintersection: f32;
        let mut dist_to_next_yintersection: f32;

        let mut x_grid_index: i32; // the current cell that the ray is in
        let mut y_grid_index: i32;

        let mut dist_to_vertical_grid_being_hit: f32; // the distance of the x and y ray intersections from
        let mut dist_to_horizontal_grid_being_hit: f32; // the viewpoint

        let mut cast_arc: i32;
        //let debug = false;

        cast_arc = self.f_player_arc;
        // field of view is 60 degree with the point of view (player's direction in the middle)
        // 30  30
        //    ^
        //  \ | /
        //   \|/
        //    v
        // we will trace the rays starting from the leftmost ray
        cast_arc -= self.angle30 as i32;
        // wrap around if necessary
        if cast_arc < 0 {
            cast_arc = self.angle360 as i32 + cast_arc;
        }

        for cast_column in 0..self.projectionplanewidth {
            // Ray is between 0 to 180 degree (1st and 2nd quadrant).

            // Ray is facing down
            if cast_arc > self.angle0 as i32 && cast_arc < self.angle180 as i32 {
                // truncuate then add to get the coordinate of the FIRST grid (horizontal
                // wall) that is in front of the player (this is in pixel unit)
                // ROUNDED DOWN
                horizontal_grid = (self.f_player_y as f32 / self.tile_size as f32).floor()
                    * self.tile_size as f32
                    + self.tile_size as f32;

                // compute distance to the next horizontal wall
                dist_to_next_horizontal_grid = self.tile_size as f32;

                let xtemp = self.f_itan_table[cast_arc as usize]
                    * (horizontal_grid - self.f_player_y as f32) as f32;
                // we can get the vertical distance to that wall by
                // (horizontalGrid-playerY)
                // we can get the horizontal distance to that wall by
                // 1/tan(arc)*verticalDistance
                // find the x interception to that wall
                x_intersection = xtemp + self.f_player_x as f32;
            }
            // Else, the ray is facing up
            else {
                horizontal_grid =
                    (self.f_player_y / self.tile_size as f32).floor() * self.tile_size as f32;
                dist_to_next_horizontal_grid = -(self.tile_size as f32);

                let xtemp = self.f_itan_table[cast_arc as usize]
                    * (horizontal_grid - self.f_player_y as f32) as f32;
                x_intersection = xtemp + self.f_player_x as f32;

                horizontal_grid -= 1.0;
            }
            // LOOK FOR HORIZONTAL WALL

            // If ray is directly facing right or left, then ignore it
            if cast_arc == self.angle0 as i32 || cast_arc == self.angle180 as i32 {
                dist_to_horizontal_grid_being_hit = f32::MAX;
            }
            // else, move the ray until it hits a horizontal wall
            else {
                dist_to_next_xintersection = self.f_xstep_table[cast_arc as usize];
                loop {
                    x_grid_index = (x_intersection / self.tile_size as f32).floor() as i32;
                    y_grid_index = (horizontal_grid as f32 / self.tile_size as f32).floor() as i32;
                    let map_index = y_grid_index * self.map_width as i32 + x_grid_index;

                    // If we've looked as far as outside the map range, then bail out
                    if x_grid_index >= self.map_width as i32
                        || y_grid_index >= self.map_height as i32
                        || x_grid_index < 0
                        || y_grid_index < 0
                    {
                        dist_to_horizontal_grid_being_hit = f32::MAX;
                        break;
                    }
                    // If the grid is not an Opening, then stop
                    else if self.f_map.chars().nth(map_index as usize).unwrap() != 'O' {
                        dist_to_horizontal_grid_being_hit = (x_intersection
                            - self.f_player_x as f32)
                            * self.f_icos_table[cast_arc as usize];
                        break;
                    }
                    // Else, keep looking.  At this point, the ray is not blocked, extend the ray to the next grid
                    else {
                        x_intersection += dist_to_next_xintersection;
                        horizontal_grid += dist_to_next_horizontal_grid;
                    }
                }
            }

            // FOLLOW X RAY
            if cast_arc < self.angle90 as i32 || cast_arc > self.angle270 as i32 {
                vertical_grid = self.tile_size as f32
                    + (self.f_player_x as f32 / self.tile_size as f32).floor()
                        * self.tile_size as f32;
                dist_to_next_vertical_grid = self.tile_size as f32;

                let ytemp = self.f_tan_table[cast_arc as usize]
                    * (vertical_grid - self.f_player_x as f32) as f32;
                y_intersection = ytemp + self.f_player_y as f32;
            }
            // RAY FACING LEFT
            else {
                vertical_grid = (self.f_player_x as f32 / self.tile_size as f32).floor()
                    * self.tile_size as f32;
                dist_to_next_vertical_grid = -(self.tile_size as f32);

                let ytemp = self.f_tan_table[cast_arc as usize]
                    * (vertical_grid as f32 - self.f_player_x as f32);
                y_intersection = ytemp + self.f_player_y as f32;

                vertical_grid -= 1.0;
            }
            // LOOK FOR VERTICAL WALL
            if cast_arc == self.angle90 as i32 || cast_arc == self.angle270 as i32 {
                dist_to_vertical_grid_being_hit = f32::MAX;
            } else {
                dist_to_next_yintersection = self.f_ystep_table[cast_arc as usize];
                loop {
                    // compute current map position to inspect
                    x_grid_index = (vertical_grid as f32 / self.tile_size as f32).floor() as i32;
                    y_grid_index = (y_intersection as f32 / self.tile_size as f32).floor() as i32;

                    let map_index = y_grid_index * self.map_width as i32 + x_grid_index;

                    if x_grid_index >= self.map_width as i32
                        || y_grid_index >= self.map_height as i32
                        || x_grid_index < 0
                        || y_grid_index < 0
                    {
                        dist_to_vertical_grid_being_hit = f32::MAX;
                        break;
                    } else if self.f_map.chars().nth(map_index as usize).unwrap() != 'O' {
                        dist_to_vertical_grid_being_hit = (y_intersection as f32
                            - self.f_player_y as f32)
                            * self.f_isin_table[cast_arc as usize];
                        break;
                    } else {
                        y_intersection += dist_to_next_yintersection;
                        vertical_grid += dist_to_next_vertical_grid;
                    }
                }
            }

            // DRAW THE WALL SLICE
            //let mut scale_factor: f32;
            let mut dist: f32;
            let x_offset;
            let top_of_wall: f32; // used to compute the top and bottom of the sliver that
            let bottom_of_wall: f32; // will be the staring point of floor and ceiling
                                     // determine which ray strikes a closer wall.
                                     // if yray distance to the wall is closer, the yDistance will be shorter than
                                     // the xDistance
            let mut is_vertical_hit = false;
            //let mut distorted_distance = 0.0;

            if dist_to_horizontal_grid_being_hit < dist_to_vertical_grid_being_hit {
                // the next function call (drawRayOnMap()) is not a part of raycating rendering part,
                // it just draws the ray on the overhead map to illustrate the raycasting process
                self.draw_ray_on_overhead_map(x_intersection, horizontal_grid, 0, 255, 0, 255);
                dist = dist_to_horizontal_grid_being_hit / self.f_fish_table[cast_column as usize];
                //				dist_y /= convert_to_float(GLfishTable[GLcastColumn]);
                //distorted_distance = dist;
                let ratio = self.f_player_distance_to_the_projection_plane as f32 / dist;
                bottom_of_wall =
                    ratio * self.f_player_height as f32 + self.f_projection_plane_ycenter as f32;
                let scale: f32 = self.f_player_distance_to_the_projection_plane as f32
                    * self.wall_height as f32
                    / dist;
                top_of_wall = bottom_of_wall - scale;
                x_offset = x_intersection % self.tile_size as f32;
            }
            // else, we use xray instead (meaning the vertical wall is closer than
            //   the horizontal wall)
            else {
                is_vertical_hit = true;
                // the next function call (drawRayOnMap()) is not a part of raycating rendering part,
                // it just draws the ray on the overhead map to illustrate the raycasting process
                self.draw_ray_on_overhead_map(vertical_grid, y_intersection, 0, 0, 255, 255);
                dist = dist_to_vertical_grid_being_hit as f32
                    / self.f_fish_table[cast_column as usize];

                x_offset = y_intersection % self.tile_size as f32;

                let ratio = self.f_player_distance_to_the_projection_plane as f32 / dist;
                bottom_of_wall =
                    ratio * self.f_player_height as f32 + self.f_projection_plane_ycenter as f32;
                let scale: f32 = self.f_player_distance_to_the_projection_plane as f32
                    * self.wall_height as f32
                    / dist;
                top_of_wall = bottom_of_wall - scale;
            }

            // Add simple shading so that farther wall slices appear darker.
            // use arbitrary value of the farthest distance.
            dist = dist.floor();

            // Trick to give different shades between vertical and horizontal (you could also use different textures for each if you wish to)
            if is_vertical_hit {
                self.draw_wall_slice_rectangle_tinted(
                    cast_column as f32,
                    top_of_wall,
                    1.0,
                    (bottom_of_wall - top_of_wall) + 1.0,
                    x_offset,
                    self.base_light_value as f32 / dist,
                );
            } else {
                self.draw_wall_slice_rectangle_tinted(
                    cast_column as f32,
                    top_of_wall,
                    1.0,
                    (bottom_of_wall - top_of_wall) + 1.0,
                    x_offset,
                    (self.base_light_value as f32 - 50.0) / dist,
                );
            }

            let bytes_per_pixel = 4;
            let projection_plane_center_y = self.f_projection_plane_ycenter;
            let last_bottom_of_wall: f32 = bottom_of_wall.floor();
            let last_top_of_wall: f32 = top_of_wall.floor();

            //*************
            // FLOOR CASTING at the simplest!  Try to find ways to optimize this, you can do it!
            //*************
            if self.assets.textures.contains_key("/images/floortile.ff") {
                // find the first bit so we can just add the width to get the
                // next row (of the same column)
                let mut target_index: i32 =
                    last_bottom_of_wall as i32 * (self.width * 1) as i32 + (1 * cast_column) as i32;
                for row in last_bottom_of_wall as i32..self.projectionplaneheight as i32 {
                    let straight_distance = self.f_player_height as f32
                        / (row as f32 - projection_plane_center_y as f32)
                        * self.f_player_distance_to_the_projection_plane as f32;

                    let actual_distance: f32 =
                        straight_distance * self.f_fish_table[cast_column as usize];

                    let mut y_end: i32 =
                        (actual_distance * self.f_sin_table[cast_arc as usize]).floor() as i32;
                    let mut x_end: i32 =
                        (actual_distance * self.f_cos_table[cast_arc as usize]).floor() as i32;

                    // Translate relative to viewer coordinates:
                    x_end = x_end.wrapping_add(self.f_player_x as i32);
                    y_end = y_end.wrapping_add(self.f_player_y as i32);

                    // Get the tile intersected by ray:
                    let cell_x: i32 = (x_end as f32 / self.tile_size as f32).floor() as i32;
                    let cell_y: i32 = (y_end as f32 / self.tile_size as f32).floor() as i32;
                    //println!("cell_x="+cell_x+" cell_y="+cell_y);

                    //Make sure the tile is within our map
                    if cell_x < self.map_width as i32
                        && cell_y < self.map_height as i32
                        && cell_x >= 0
                        && cell_y >= 0
                    {
                        if target_index > 0 {
                            // Find offset of tile and column in texture
                            let tile_row = (y_end as f32 % self.tile_size as f32).floor() as i32;
                            let tile_column = (x_end as f32 % self.tile_size as f32).floor() as i32;
                            // Pixel to draw
                            let source_index = (tile_row as u32
                                * self.assets.textures["/images/floortile.ff"].width
                                * bytes_per_pixel)
                                + (bytes_per_pixel * tile_column as u32);

                            // Cheap shading trick
                            let brightness_level = 150.0 / actual_distance;
                            let red = clamp_u16_to_u8(
                                self.assets.textures["/images/floortile.ff"].data
                                    [source_index as usize],
                            ) as f32
                                * brightness_level;
                            let green = clamp_u16_to_u8(
                                self.assets.textures["/images/floortile.ff"].data
                                    [source_index as usize + 1],
                            ) as f32
                                * brightness_level;
                            let blue = clamp_u16_to_u8(
                                self.assets.textures["/images/floortile.ff"].data
                                    [source_index as usize + 2],
                            ) as f32
                                * brightness_level;
                            let alpha = clamp_u16_to_u8(
                                self.assets.textures["/images/floortile.ff"].data
                                    [source_index as usize + 3],
                            );

                            // Draw the pixel
                            self.canvas[target_index as usize] = u8_to_color(
                                alpha,
                                red.floor() as u8,
                                green.floor() as u8,
                                blue.floor() as u8,
                            );
                        }

                        // Go to the next pixel (directly under the current pixel)
                        target_index += (1 * self.width) as i32;
                    }
                }
            }
            //*************
            // CEILING CASTING at the simplest!  Try to find ways to optimize this, you can do it!
            //*************
            if !self.no_ceiling && self.assets.textures.contains_key("/images/tile41.ff") {
                // find the first bit so we can just add the width to get the
                // next row (of the same column)

                let mut target_index: i32 =
                    last_top_of_wall as i32 * (self.width * 1) as i32 + (1 * cast_column) as i32;
                for row in (0..=last_top_of_wall as i32).rev() {
                    let ratio: f32 = (self.wall_height as i32 - self.f_player_height) as f32
                        / (projection_plane_center_y - row as i32) as f32;

                    let diagonal_distance = (self.f_player_distance_to_the_projection_plane as f32
                        * ratio
                        * self.f_fish_table[cast_column as usize])
                        .floor();

                    let mut y_end: i32 =
                        (diagonal_distance * self.f_sin_table[cast_arc as usize]).floor() as i32;
                    let mut x_end: i32 =
                        (diagonal_distance * self.f_cos_table[cast_arc as usize]).floor() as i32;

                    // Translate relative to viewer coordinates:
                    x_end = x_end.wrapping_add(self.f_player_x as i32);
                    y_end = y_end.wrapping_add(self.f_player_y as i32);

                    // Get the tile intersected by ray:
                    let cell_x: i32 = (x_end as f32 / self.tile_size as f32).floor() as i32;
                    let cell_y: i32 = (y_end as f32 / self.tile_size as f32).floor() as i32;
                    //println!("cell_x="+cell_x+" cell_y="+cell_y);

                    //Make sure the tile is within our map
                    if cell_x < self.map_width as i32
                        && cell_y < self.map_height as i32
                        && cell_x >= 0
                        && cell_y >= 0
                    {
                        // Find offset of tile and column in texture
                        let tile_row: i32 = (y_end as f32 % self.tile_size as f32).floor() as i32;
                        let tile_column: i32 =
                            (x_end as f32 % self.tile_size as f32).floor() as i32;
                        // Pixel to draw
                        let source_index = (tile_row as u32
                            * self.assets.textures["/images/tile41.ff"].width
                            * bytes_per_pixel)
                            + (bytes_per_pixel * tile_column as u32);
                        //println!("sourceIndex="+sourceIndex);
                        // Cheap shading trick
                        let brightness_level = 100.0 / diagonal_distance;
                        let red = clamp_u16_to_u8(
                            self.assets.textures["/images/tile41.ff"].data[source_index as usize],
                        ) as f32
                            * brightness_level;
                        let green = clamp_u16_to_u8(
                            self.assets.textures["/images/tile41.ff"].data
                                [source_index as usize + 1],
                        ) as f32
                            * brightness_level;
                        let blue = clamp_u16_to_u8(
                            self.assets.textures["/images/tile41.ff"].data
                                [source_index as usize + 2],
                        ) as f32
                            * brightness_level;
                        let alpha = clamp_u16_to_u8(
                            self.assets.textures["/images/tile41.ff"].data
                                [source_index as usize + 3],
                        );

                        // Draw the pixel
                        self.canvas[target_index as usize] = u8_to_color(
                            alpha,
                            red.floor() as u8,
                            green.floor() as u8,
                            blue.floor() as u8,
                        );

                        // Go to the next pixel (directly above the current pixel)
                        target_index -= (1 * self.width) as i32;
                    }
                }
            }

            // TRACE THE NEXT RAY
            cast_arc += 1;
            if cast_arc >= self.angle360 as i32 {
                cast_arc -= self.angle360 as i32;
            }
        }
    }

    // This function is called every certain interval (see self.frameRate) to handle input and render the screen
    fn update(&mut self) {
        self.clear_offscreen_canvas();

        if self.no_ceiling {
            self.draw_background();
        }
        self.raycast();
        self.draw_overhead_map();
        self.draw_player_pov_on_overhead_map(0, 0);
        //self.blitOffscreenCanvas(); //we are writting directly to the buffer, then we copy. no need for this
        let mut player_arc_delta: i32 = 0;

        if self.f_key_left {
            self.f_player_arc -= self.angle5 as i32;
            player_arc_delta = -(self.angle5 as i32);
            if self.f_player_arc < self.angle0 as i32 {
                self.f_player_arc += self.angle360 as i32;
            }
        }
        // rotate right
        else if self.f_key_right {
            self.f_player_arc += self.angle5 as i32;
            player_arc_delta = self.angle5 as i32;
            if self.f_player_arc >= self.angle360 as i32 {
                self.f_player_arc -= self.angle360 as i32;
            }
        }
        self.f_background_image_arc -= player_arc_delta;
        if self.no_ceiling && self.assets.textures.contains_key("/images/bgr.ff") {
            //we are not drawing the background yet
            // This code wraps around the background image so that it can be drawn just one.
            // For this to work, the first section of the image needs to be repeated on the third section (see the image used in this example)
            if self.f_background_image_arc < -(self.projectionplanewidth as i32) * 2 {
                self.f_background_image_arc =
                    self.projectionplanewidth as i32 * 2 + self.f_background_image_arc;
            } else if self.f_background_image_arc > 0 {
                self.f_background_image_arc = -(self.assets.textures["/images/bgr.ff"].width
                    as i32
                    - self.projectionplanewidth as i32
                    - (self.f_background_image_arc));
            }
        }
        //  _____     _
        // |\ arc     |
        // |  \       y
        // |    \     |
        //            -
        // |--x--|
        //
        //  sin(arc)=y/diagonal
        //  cos(arc)=x/diagonal   where diagonal=speed
        let player_xdir: f32 = self.f_cos_table[self.f_player_arc as usize];
        let player_ydir: f32 = self.f_sin_table[self.f_player_arc as usize];

        let mut dx: f32 = 0.0;
        let mut dy: f32 = 0.0;
        // move forward
        if self.f_key_up {
            dx = (player_xdir * self.f_player_speed as f32).round();
            dy = (player_ydir * self.f_player_speed as f32).round();
        }
        // move backward
        else if self.f_key_down {
            dx = -(player_xdir * self.f_player_speed as f32).round();
            dy = -(player_ydir * self.f_player_speed as f32).round();
        }
        self.f_player_x += dx;
        self.f_player_y += dy;

        // compute cell position
        let player_xcell = (self.f_player_x / self.tile_size as f32).floor();
        let player_ycell = (self.f_player_y / self.tile_size as f32).floor();

        // compute position relative to cell (ie: how many pixel from edge of cell)
        let player_xcell_offset = self.f_player_x % self.tile_size as f32;
        let player_ycell_offset = self.f_player_y % self.tile_size as f32;

        let min_distance_to_wall = 30.0;

        // make sure the player don't bump into walls
        if dx > 0.0 {
            // moving right
            if self
                .f_map
                .chars()
                .nth(
                    ((player_ycell as i32 * self.map_width as i32) + player_xcell as i32 + 1)
                        as usize,
                )
                .unwrap()
                != 'O'
                && player_xcell_offset > (self.tile_size as f32 - min_distance_to_wall)
            {
                // back player up
                self.f_player_x -=
                    player_xcell_offset - (self.tile_size as f32 - min_distance_to_wall);
            }
        } else {
            // moving left
            if self
                .f_map
                .chars()
                .nth(
                    ((player_ycell as i32 * self.map_width as i32) + player_xcell as i32 - 1)
                        as usize,
                )
                .unwrap()
                != 'O'
                && player_xcell_offset < (min_distance_to_wall)
            {
                // back player up
                self.f_player_x += min_distance_to_wall - player_xcell_offset;
            }
        }

        if dy < 0.0 {
            // moving up
            if self
                .f_map
                .chars()
                .nth(
                    (((player_ycell as i32 - 1) * self.map_width as i32) + player_xcell as i32)
                        as usize,
                )
                .unwrap()
                != 'O'
                && player_ycell_offset < (min_distance_to_wall)
            {
                // back player up
                self.f_player_y += min_distance_to_wall - player_ycell_offset;
            }
        } else {
            // moving down
            if self
                .f_map
                .chars()
                .nth(
                    (((player_ycell as i32 + 1) * self.map_width as i32) + player_xcell as i32)
                        as usize,
                )
                .unwrap()
                != 'O'
                && player_ycell_offset > (self.tile_size as f32 - min_distance_to_wall)
            {
                // back player up
                self.f_player_y -=
                    player_ycell_offset - (self.tile_size as f32 - min_distance_to_wall);
            }
        }

        if self.f_key_look_up {
            self.f_projection_plane_ycenter += 15;
        } else if self.f_key_look_down {
            self.f_projection_plane_ycenter -= 15;
        }

        if self.f_projection_plane_ycenter < -(self.projectionplaneheight as i32) {
            self.f_projection_plane_ycenter = -(self.projectionplaneheight as i32);
        } else if self.f_projection_plane_ycenter
            >= (self.projectionplaneheight as f32 * 1.5) as i32
        {
            self.f_projection_plane_ycenter =
                (self.projectionplaneheight as f32 * 1.5 - 1.0) as i32;
        }

        if self.f_key_fly_up {
            self.f_player_height += 1;
        } else if self.f_key_fly_down {
            self.f_player_height -= 1;
        }

        if self.f_player_height < -5 {
            //originally -5
            self.f_player_height = -5;
        } else if self.f_player_height > self.wall_height as i32 - 5 {
            self.f_player_height = self.wall_height as i32 - 5;
        }

        if self.f_key_ceiling_toggle {
            self.no_ceiling = !self.no_ceiling;
        }
    }

    fn handle_keys(&mut self, window: &Window) {
        // UP keypad
        self.f_key_up = window.is_key_down(Key::W);

        // DOWN keypad
        self.f_key_down = window.is_key_down(Key::S);

        // LEFT keypad
        self.f_key_left = window.is_key_down(Key::A);

        // RIGHT keypad
        self.f_key_right = window.is_key_down(Key::D);

        // LOOK UP
        self.f_key_look_up = window.is_key_down(Key::Q);

        // LOOK DOWN
        self.f_key_look_down = window.is_key_down(Key::Z);

        // FLY UP
        self.f_key_fly_up = window.is_key_down(Key::E);

        // FLY DOWN
        self.f_key_fly_down = window.is_key_down(Key::C);

        // CEILING
        self.f_key_ceiling_toggle = window.is_key_down(Key::F); //we should ideally have some
    }

    /*    fn flip_buffer_in_use(&mut self) {
        if self.buffer_n == 0 {
            self.buffer_n = 1;
        } else {
            self.buffer_n = 0;
        }
    }*/

    /**
     * return a slice in the buffer that has just been updated
     */
    pub fn get_buffer_to_print(&mut self) -> &[u32] {
        let start_offset = self.buffer_n * self.area_size as usize;
        &self.canvas[start_offset..start_offset + self.area_size as usize]
    }

    pub fn game_step(&mut self, window: &Window) {
        /*
        self.flip_buffer_in_use(); // we are not using the two buffers in the other part of the code
        let offset = self.buffer_n * self.area_size as usize;

        // clear the buffer
        // nah. we are drawing the entire screen anyway
        self.canvas
            .iter_mut()
            //.skip(offset)
            .take(self.area_size as usize)
            .for_each(|value| *value = 0xFF01A101); // clear in blue, so we can see if we are drawing something
        */
        self.handle_keys(&window);
        self.update();
    }
}
