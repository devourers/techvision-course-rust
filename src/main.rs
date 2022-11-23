use itertools::Itertools;
use num::traits::Pow;


static COLORS: &'static [f64] =
 &[1.0, 1.0, 1.0, 
  1.0, 1.0, 1.0, 
  1.0, 1.0, 1.0];
static ALBEDO: &'static [f64] = 
&[0.7, 0.9, 0.1, 
  0.9, 0.5, 0.4, 
  0.6, 0.1, 1.0];
static SIZE: usize = 200;
static LIGHT_LUMINOSITY: f64 = 1.0;
static DIAG: f64 = (SIZE*3 * SIZE * 3 + SIZE*3 * SIZE*3) as f64;
static ITER_TAKE: usize = 5000;
const NTHREADS: usize = 12;
static LOCATIONS: &'static [(usize, usize)] = 
&[(0, 0), 
  (0, 1), 
  (0, 2), 
  (1, 0), 
  (1, 1), 
  (1, 2), 
  (2, 0), 
  (2, 1),
  (2, 2)];
static DIRECTIONS: &'static [(i64, i64)] = &[
 (-1, 1), 
 (1, 1),
 (1, 0), 
 (0, 1), 
 (-1, 0), 
 (0, -1), 
 (-1, -1)
];

fn main(){
    let sim_app = LightSimApp::init(SIZE);
    let options = eframe::NativeOptions::default();
    eframe::run_native("LightSim", options, Box::new(|_cc| Box::new(sim_app)));
}

fn eucl_dist(a: &(usize, usize), b: &(usize, usize)) -> f64{
    let a_ = (a.0 as f64, a.1 as f64);
    let b_ = (b.0 as f64, b.1 as f64);
    let mut dist = (b_.0 - a_.0) * (b_.0 - a_.0) + (b_.1 - a_.1) * (b_.1 - a_.1);
    dist = dist.sqrt();
    return dist;
}

fn get_circle_center(x1: &(usize, usize), x2: &(usize, usize), x3: &(usize, usize)) -> (bool, (usize, usize)){
    //x0
    //count y_bracket once, multiply in up 
    let mut y1bracket = (x2.0 * x2.0 + x2.1 * x2.1) as i64 - (x3.0 * x3.0) as i64 - (x3.1 * x3.1) as i64;
    y1bracket *= x1.1 as i64;
    let mut y2bracket = (x3.0 * x3.0 + x3.1 * x3.1) as i64 - (x1.0 * x1.0) as i64 - (x1.1 * x1.1) as i64;
    y2bracket *= x2.1 as i64;
    let mut y3bracket = (x1.0 * x1.0 + x1.1 * x1.1) as i64 - (x2.0 * x2.0) as i64 - (x2.1 * x2.1) as i64;
    y3bracket *= x3.1 as i64;
    let up = y1bracket + y2bracket + y3bracket;
    let down = x1.0 as i64 * (x2.1 as i64 - x3.1 as i64) + x2.0 as i64*(x3.1 as i64 - x1.1 as i64) + x3.0 as i64*(x1.1 as i64 - x2.1 as i64);
    let mut x = (up as f64) / (down as f64);
    x *= -0.5;
    //y0
    let mut y1bracket = (x2.0 * x2.0 + x2.1 * x2.1) as i64 - (x3.0 * x3.0) as i64 - (x3.1 * x3.1) as i64;
    y1bracket *= x1.0 as i64;
    let mut y2bracket = (x3.0 * x3.0 + x3.1 * x3.1) as i64 - (x1.0 * x1.0) as i64 - (x1.1 * x1.1) as i64;
    y2bracket *= x2.0 as i64;
    let mut y3bracket = (x1.0 * x1.0 + x1.1 * x1.1) as i64 - (x2.0 * x2.0) as i64 - (x2.1 * x2.1) as i64;
    y3bracket *= x3.0 as i64;
    let up = y1bracket + y2bracket + y3bracket;
    let mut y = (up as f64) / (down as f64);
    y *= 0.5;
    //check if eligable
    let mut approved = true;
    if x <= -0.5 || y <= -0.5 || x >= 599.5 || y >= 599.5{
        approved = false;
    }
    let x_ = x.round() as usize;
    let y_ = y.round() as usize;
    return (approved, (x_, y_));
}

fn process_patch(cluster: std::collections::HashMap<usize, Vec<(usize, usize)>>) -> std::collections::HashMap<(usize, usize), usize>{
    let mut children = vec!();
    let mut answers: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    //mapreduce
    for cl in cluster{
        children.push(std::thread::spawn(move || -> std::collections::HashMap<(usize, usize), usize> {
            let mut cur_answers: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
            let points = cl.1;
            let it = points.iter().combinations(3).take(ITER_TAKE);
            it.for_each(|i|{
                let cur_ans = get_circle_center(i[0], i[1], i[2]);
                if cur_ans.0 == true{
                    if cur_answers.contains_key(&cur_ans.1){
                        *cur_answers.get_mut(&cur_ans.1).unwrap() += 1;
                    }
                    else{
                        cur_answers.insert(cur_ans.1, 1);
                    }
                }
            });
            return cur_answers;
        }));
    }
    let final_result = 
    children.into_iter().map(|c| c.join().unwrap());
    for a in final_result{
        for k in a.keys(){
            if answers.contains_key(k){
                *answers.get_mut(k).unwrap() += 1;
            }
            else{
                answers.insert(*k, *a.get(k).unwrap());
            }
        }
    }
    return answers;
}


struct LightSimApp{
    light_source: LightSource,
    scene: Scene,
    img_gui: egui_extras::RetainedImage,
    reverse_solution_height: u32,
    revere_solution_albedo: Vec<u8>,
    reverse_solution_location: (usize, usize),
    scene_arr: ndarray::Array2::<f64>
}

fn load_im_egui() -> eframe::epaint::ColorImage{
    let img = image::open("scene.png").unwrap().to_rgba8();
    let rgba = img.as_raw();
    let img_ = eframe::epaint::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], rgba);
    return img_;
}

fn solve_eq(ls: (usize, usize), center: (usize, usize), edge: (usize, usize), scene_arr: &ndarray::Array2::<f64>) -> (f64, f64){
    let b2 = scene_arr[[center.0, center.1]].powf(1.0 / 3.0);
    let b1 = scene_arr[[edge.0, edge.1]].powf(1.0 / 3.0);
    let diff = b2 - b1;
    let r1 = eucl_dist(&ls, &edge);
    let r2 = eucl_dist(&ls, &center);
    let up = b2 * b2 * r2 * r2 - b1 * b1 * r1 * r1;
    let down = (b1 * b1 - b2 * b2) as f64;
    let mut h = up.abs() / down.abs();
    h = h.sqrt();
    return (h, diff.abs());
}

fn within_bound(loc: (i64, i64)) -> bool{
    if loc.0 < 0 || loc.1 < 0 || loc.0 > (SIZE*3 - 1) as i64 || loc.1 > (SIZE*3 - 1) as i64{
        return false;
    }
    else{
        return true;
    }
}

fn get_patch(j: usize, k: usize) -> (usize, usize){
    let mut x = k / SIZE;
    let mut y = j / SIZE;
    if x > 2{
        x = 2;
    }
    if y > 2{
        y = 2;
    }
    return (x, y);
}

impl LightSimApp{
    fn init(sz: usize) -> Self{
        let ls = LightSource::init();
        let sc = Scene::init(sz);
        let img_ = load_im_egui();
        let rev_sol_h = 0;
        let rev_sol_loc = (0, 0);
        let rev_sol_albed: Vec<u8> = [0, 0, 0, 
                                       0, 0, 0, 
                                       0, 0, 0].to_vec();
        let shape = (sz*3, sz*3);
        let arr = ndarray::Array2::<f64>::default(shape);
        return LightSimApp { 
            light_source: ls, 
            scene: sc,
            reverse_solution_location: rev_sol_loc,
            img_gui: egui_extras::RetainedImage::from_color_image("sceneimg", img_),
            reverse_solution_height: rev_sol_h,
            revere_solution_albedo: rev_sol_albed,
            scene_arr: arr
        }
    }

    fn clusterize_patch(&mut self, loc: &(usize, usize)) -> (bool, std::collections::HashMap<usize, Vec<(usize, usize)>>){
        let loc_restrictions = (loc.1 * SIZE, loc.0*SIZE);
        let patch = self.scene_arr.slice(ndarray::s![loc_restrictions.0..loc_restrictions.0+SIZE,loc_restrictions.1..loc_restrictions.1+SIZE]);
        let (min, max) = find_min_max(&patch.to_owned());
        let mut clusters: std::collections::HashMap<usize, Vec<(usize, usize)>> = std::collections::HashMap::new();
        if (min - max).abs() < 0.1{
            return (false, clusters);
        }
        let shape = patch.shape();
        for i in 0..shape[0]{
            for j in 0..shape[1]{
                let current_brightness = (patch[[i, j]] * 100000000.0) as usize;
                if clusters.contains_key(&current_brightness){
                    clusters.get_mut(&current_brightness).unwrap().push((loc.1 * SIZE + i, loc.0 * SIZE + j));
                }
                else if  clusters.keys().len()< 5 * NTHREADS{
                    clusters.insert(current_brightness, Vec::new());
                }
            }
        }
        return (true, clusters);
    }

    fn update_(&mut self){
        self.light_source.generate_light_matrix();
        self.scene_arr = self.scene.update(&self.light_source);
        self.solve_loc();
        self.solve_height();
        self.solve_albedo();
        let img_ = load_im_egui();
        self.img_gui = egui_extras::RetainedImage::from_color_image("sceneimg", img_);
    }


    fn solve_loc(&mut self){
        let mut answers: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
        for loc in LOCATIONS{
            let (valid, clusters) = self.clusterize_patch(loc);
            if valid{
                let pts = process_patch(clusters);
                for pt in pts{
                    if answers.contains_key(&pt.0){
                        *answers.get_mut(&pt.0).unwrap() += pt.1;
                    }
                    else{
                        answers.insert(pt.0, pt.1);
                    }
                }
            }
        }
        let max_elem = answers.iter().max_by_key(|entry| entry.1);
        if !max_elem.is_none(){
            self.reverse_solution_location = *max_elem.unwrap().0;
        }
    }

    fn launch_ray(&mut self, direction: &(i64, i64)) -> (f64, f64){
        let mut res = (0.0, 0.0); //height, diff
        let mut curr_pos = (self.reverse_solution_location.0 as i64, self.reverse_solution_location.1 as i64);
        let mut mov_pos = curr_pos;
        while within_bound(mov_pos){
            let new_pos = (mov_pos.0 + direction.0, mov_pos.1 + direction.1);
            if within_bound(new_pos){
                if get_patch(new_pos.0 as usize, new_pos.1 as usize) == 
                get_patch(curr_pos.0 as usize, curr_pos.1 as usize){
                    let cur_h = solve_eq(self.reverse_solution_location, (curr_pos.0 as usize, curr_pos.1 as usize),
                                                    (new_pos.0 as usize, new_pos.1 as usize), 
                                                    &self.scene_arr);
                    if cur_h.1 > res.1{
                        res = cur_h;
                    }
                }
                else{
                    curr_pos = new_pos;
                }
                mov_pos = (mov_pos.0 + direction.0, mov_pos.1 + direction.1);
            }
            else{
                break;
            }
        }
        return res;
    }


    fn solve_height(&mut self){
        let mut h_vec: Vec<(f64, f64)> = vec!();
        //rewrite launch ray to non-self, parallel this
        for dir in DIRECTIONS{
            let h_c = self.launch_ray(dir);
            h_vec.push(h_c);
        }
        h_vec.sort_by(|a, b| a.1.total_cmp(&b.1));
        self.reverse_solution_height = h_vec[h_vec.len() - 1].0.round() as u32;
    }

    fn solve_albedo(&mut self){
        //TODO
        //count diff on border of two squares for each pair of squares, recount from here.
    }
    
}

struct LightSource{
    location: (usize, usize),
    coordinates: (usize, usize),
    height: u32, //in pixels
    is_on: bool,
    size: usize,
    light_matrix: ndarray::Array2::<f64>
}


fn get_actual_location(coordinates: (usize, usize), location: (usize, usize), size: usize) -> (usize, usize){
    return (location.1 * size + coordinates.0, location.0 * size +coordinates.1);
}


fn get_light(coordinates: (usize, usize), location: (usize, usize), height: u32, size: usize, j: usize, k: usize) -> f64{
    let actual_location = get_actual_location(coordinates, location, size);
    let ground_dist = eucl_dist(&actual_location, &(j, k));
    let tg_a = ground_dist / (height as f64);
    let alpha = tg_a.atan();
    let mut x = k / size;
    let mut y = j / size;
    if x > 2{
        x = 2;
    }
    if y > 2{
        y = 2;
    }
    return alpha.cos().pow(3) * LIGHT_LUMINOSITY * ALBEDO[3 * x + y];
}

impl LightSource{
    fn init() -> Self{
        let location_ = (0, 0);
        let height_: u32 = 0;
        let shape = (SIZE*3, SIZE*3);
        let light_matrix_ = ndarray::Array2::<f64>::default(shape);
        let is_on_ = false;
        return LightSource { 
            location: location_,
            coordinates: (0, 0),
            height: height_, 
            size: SIZE,
            light_matrix: light_matrix_,
            is_on: is_on_ 
        };
    }

    fn generate_light_matrix(&mut self){
        if self.is_on{
            ndarray::Zip::indexed(self.light_matrix.outer_iter_mut()).par_for_each(|j, mut row| {
                for (k, col) in row.iter_mut().enumerate(){
                    *col = get_light(self.coordinates, self.location, self.height, self.size, j, k);
                }
            });
        }
    }

}

struct Scene{
    scene_array: ndarray::Array2::<f64>,
    scene_image: image::GrayImage,
    size: usize
}

fn decide(sz: usize, j: usize, k:usize) -> f64{
    let mut x = k / sz;
    let mut y = j / sz;
    if x > 2{
        x = 2;
    }
    if y > 2{
        y = 2;
    }
    return COLORS[x * 3 +  y] as f64;
}

fn decide_light(orig: f64, lighted: f64) -> f64{
    return orig*lighted;
}

fn clinear_to_srgb(input: f64) -> f64{
    let a: f64 = 0.055;
    if input <= 0.0031308{
        return 12.92 * input;
    }
    else{
        let r1 = input.powf(1.0 / 2.4);
        let r2 = (1.0 + a) * r1;
        let r3 = r2 - a;
        return r3;
    }
}

fn srgb_to_clinear(input: usize) -> f64{
    let a: f64 = 0.055;
    let input_01 = (input as f64) / 255.0;
    if input <= 11{
        return input_01 / 12.92;
    }
    else{
        let r1 = input_01 + a;
        let r2 = r1 / (1.0 + a);
        let r3 = r2.powf(2.4);
        return r3;
    }
}

fn generate_arr(sz: usize) -> ndarray::Array2::<f64>{
    let shape = (sz*3, sz*3);
    let mut arr = ndarray::Array2::<f64>::default(shape);
    ndarray::Zip::indexed(arr.outer_iter_mut()).par_for_each(|j, mut row| {
        for (k, col) in row.iter_mut().enumerate(){
            *col = decide(sz, j, k);
        }
    });
    return arr
}


fn find_min_max(arr: &ndarray::Array2::<f64>) -> (f64, f64){
    let shape =arr.shape();
    let mut min = arr[[0, 0]];
    let mut max = arr[[0, 0]];
    for i in 0..shape[0]{
        for j in 0..shape[1]{
            if arr[[i, j]] > max{
                max = arr[[i, j]];
            }
            if arr[[i, j]] < min{
                min = arr[[i, j]];
            }
        }
    }
    return (min, max);
}

fn prep_arr(arr: &ndarray::Array2::<f64>) -> ndarray::Array2::<f64>{
    let mut new_arr = ndarray::Array2::<f64>::default((arr.shape()[0], arr.shape()[1]));
    ndarray::Zip::indexed(new_arr.outer_iter_mut()).par_for_each(|j, mut row| {
        for (k, col) in row.iter_mut().enumerate(){
            *col = clinear_to_srgb(arr[[j, k]]) * 255.0;
        }
    });
    let (_min, max) = find_min_max(&new_arr);
    let coef = 255.0 / max;
    new_arr *= coef;
    return new_arr;
}

fn arr_to_img(arr: &ndarray::Array2::<f64>) -> image::GrayImage{
    let arr_ = prep_arr(&arr);
    let mut img = image::ImageBuffer::new(arr_.dim().0 as u32, arr_.dim().1 as u32);
    for (r, row) in arr_.outer_iter().enumerate()  {
        for (c, col) in row.iter().enumerate(){
            let pixel = (*col).round() as u8;
            let pixel = image::Luma([pixel]);
            img.put_pixel(r as u32, c as u32, pixel);
        }
    }
    return img;  
}

fn generate_arr_and_img(sz: usize)-> (ndarray::Array2::<f64>, image::GrayImage){
    let arr = generate_arr(sz);
    let img = arr_to_img(&arr);
    return (arr, img)
}


impl Scene{
    fn init(sz: usize) -> Self{
        let (arr, img) = generate_arr_and_img(sz);
        img.save("scene.png").unwrap();
        return Scene{
            scene_array: arr, 
            scene_image: img,
            size: sz
        };
    }

    fn recount_final_array(&self, light_matrix: &ndarray::Array2::<f64>) -> ndarray::Array2::<f64>{
        let shape = (self.size*3, self.size*3);
        let mut arr = ndarray::Array2::<f64>::default(shape);
        ndarray::Zip::indexed(arr.outer_iter_mut()).par_for_each(|j, mut row| {
            for (k, col) in row.iter_mut().enumerate(){
                *col = decide_light(self.scene_array[[j, k]], light_matrix[[j, k]]);
            }
        });
        return arr;
    }

    fn update(&mut self, ls: &LightSource) -> ndarray::Array2::<f64>{
        let mut new_arr = self.recount_final_array(&ls.light_matrix);
        if !ls.is_on{
            new_arr = self.scene_array.clone();
        }
        self.scene_image = arr_to_img(&new_arr);
        self.scene_image.save("scene.png").unwrap();
        return new_arr;
    }
}

//GUI
impl eframe::App for LightSimApp{
        fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame){
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Light Simulation");
            ui.vertical(|ui|{
                ui.vertical(|ui|{
                    ui.add(eframe::egui::Slider::new(&mut self.light_source.height, 0..=1200).text("Light source height"));
                    eframe::egui::ComboBox::from_label("Light Position")
                    .selected_text(format!("{:?}", self.light_source.location)).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.light_source.location, (0, 0), "(0, 0)");
                        ui.selectable_value(&mut self.light_source.location, (0, 1), "(0, 1)");
                        ui.selectable_value(&mut self.light_source.location, (0, 2), "(0, 2)");
                        ui.selectable_value(&mut self.light_source.location, (1, 0), "(1, 0)");
                        ui.selectable_value(&mut self.light_source.location, (1, 1), "(1, 1)");
                        ui.selectable_value(&mut self.light_source.location, (1, 2), "(1, 2)");
                        ui.selectable_value(&mut self.light_source.location, (2, 0), "(2, 0)");
                        ui.selectable_value(&mut self.light_source.location, (2, 1), "(2, 1)");
                        ui.selectable_value(&mut self.light_source.location, (2, 2), "(2, 2)");
                    });
                    ui.add(eframe::egui::Slider::new(&mut self.light_source.coordinates.0, 0..=SIZE-1).text("Light source X coordinate"));
                    ui.add(eframe::egui::Slider::new(&mut self.light_source.coordinates.1, 0..=SIZE-1).text("Light source Y coordinate"));
                    ui.add(eframe::egui::Checkbox::new(&mut self.light_source.is_on, "Turn the light on"));
                });
            self.img_gui.show(ui);
            if ui.button("Save pic").clicked(){
                let path = "scene_h".to_string() + self.light_source.height.to_string().as_str() + ".png";
                self.scene.scene_image.save(path).unwrap();
            }
            if self.light_source.is_on{
                self.update_();
                ui.label("Reverse task soltions:");
                ui.horizontal(|ui| {
                    ui.label(format!("location: {:?}", self.reverse_solution_location));
                    ui.label(format!(" ~ error{}", (eucl_dist(&get_actual_location(self.light_source.coordinates, self.light_source.location, SIZE), &self.reverse_solution_location) as f64 / DIAG.sqrt())));
                 });
                ui.horizontal(|ui| {
                    ui.label(format!("height: {}", (self.reverse_solution_height as f32 / DIAG.sqrt() as f32)));
                    ui.label(format!(" ~ error {}", (self.reverse_solution_height.abs_diff(self.light_source.height) as f32/ self.light_source.height as f32)));
                });
                for i in 0..3{
                    for j in 0..3{
                        ui.label(format!("Albedo [{}, {}] : {}", i, j, self.revere_solution_albedo[i + 3*j]));
                    }
                }
            }
            });
            });
    }    
}
