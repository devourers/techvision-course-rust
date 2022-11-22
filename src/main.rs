static COLORS: &'static [u8] =
 &[50, 150, 70, 
  150, 70, 60, 
  70, 60, 255];
static ALBEDO: &'static [f32] = 
&[0.3, 0.7, 0.6, 
  0.7, 0.6, 0.5, 
  0.6, 0.5, 1.0];
static SIZE: usize = 200;
static LIGHT_LUMINOSITY: f32 = 1.0;

fn main(){
    let sim_app = LightSimApp::init(SIZE);
    let options = eframe::NativeOptions::default();
    eframe::run_native("LightSim", options, Box::new(|_cc| Box::new(sim_app)));
}

fn eucl_dist(a: (usize, usize), b: (usize, usize)) -> f32{
    let a_ = (a.0 as f32, a.1 as f32);
    let b_ = (b.0 as f32, b.1 as f32);
    let mut dist = (b_.0 - a_.0) * (b_.0 - a_.0) + (b_.1 - a_.1) *(b_.1 - a_.1);
    dist = dist.sqrt();
    return dist;
}

struct LightSimApp{
    light_source: LightSource,
    scene: Scene,
    img_gui: egui_extras::RetainedImage,
    reverse_solution_height: u32,
    revere_solution_albedo: Vec<u8>,
    scene_arr: ndarray::Array2::<f32>
}

fn load_im_egui() -> eframe::epaint::ColorImage{
    let img = image::open("scene.png").unwrap().to_rgba8();
    let rgba = img.as_raw();
    let img_ = eframe::epaint::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], rgba);
    return img_;
}

fn solve_eq(center: (usize, usize), edge: (usize, usize), scene_arr: &ndarray::Array2::<f32>) -> (f32, f32){
    let b2 = /*srgb_to_clinear*/(scene_arr[[center.0, center.1]].round() as usize) as f32;
    let b1 = /*srgb_to_clinear*/(scene_arr[[edge.0, edge.1]].round() as usize) as f32;
    let diff = b2 - b1;
    let r1 = eucl_dist((0, 0), edge);
    let r2 = eucl_dist((0, 0), center);
    let up = b2 * b2 * r2 * r2 - b1 * b1 * r1 * r1;
    let down = (b1 * b1 - b2 * b2) as f32;
    let mut h = up.abs() / down.abs();
    h = h.sqrt();
    return (h, diff.abs());
}

impl LightSimApp{
    fn init(sz: usize) -> Self{
        let ls = LightSource::init();
        let sc = Scene::init(sz);
        let img_ = load_im_egui();
        let rev_sol_h = 0;
        let rev_sol_albed: Vec<u8> = [0, 0, 0, 
                                       0, 0, 0, 
                                       0, 0, 0].to_vec();
        let shape = (sz*3, sz*3);
        let arr = ndarray::Array2::<f32>::default(shape);
        return LightSimApp { 
            light_source: ls, 
            scene: sc,
            img_gui: egui_extras::RetainedImage::from_color_image("sceneimg", img_),
            reverse_solution_height: rev_sol_h,
            revere_solution_albedo: rev_sol_albed,
            scene_arr: arr
        }
    }

    fn update_(&mut self){
        self.light_source.generate_light_matrix();
        self.scene_arr = self.scene.update(&self.light_source);
        self.solve_height();
        self.solve_albedo();
        let img_ = load_im_egui();
        self.img_gui = egui_extras::RetainedImage::from_color_image("sceneimg", img_);
    }

    fn solve_height(&mut self){
        let light_loc = self.light_source.location;
        if light_loc == (0, 0){
            let h1 = solve_eq((0, 0), (199, 199), &self.scene_arr);
            let h2 = solve_eq((200, 200), (399, 399), &self.scene_arr);
            let h3 = solve_eq((400, 400), (599, 599), &self.scene_arr);
            let h4 = solve_eq((200, 0), (399, 0), &self.scene_arr);
            let h5 = solve_eq((400, 0), (599, 0), &self.scene_arr);
            let h6 = solve_eq((0, 200), (0, 399), &self.scene_arr);
            let h7 = solve_eq((0, 400), (0, 599), &self.scene_arr);
            let h_vec: Vec<(f32, f32)> = [h1, h2, h3, h4, h5, h6, h7].to_vec();
        }
    }

    fn solve_albedo(&mut self){

    }
}

struct LightSource{
    location: (usize, usize),
    height: u32, //in pixels
    is_on: bool,
    size: usize,
    light_matrix: ndarray::Array2::<f32>
}


fn get_actual_location(location: (usize, usize), size: usize) -> (usize, usize){
    if location == (0, 0){
        return (0 , 0);
    }
    else if location == (0, 2){
        return (size*3-1, 0);
    }
    else if location == (2, 0){
        return (0, size*3-1);
    }
    else if location == (2, 2){
        return (size*3-1, size*3-1);
    }
    else{
        return (location.1 * size + size/2, location.0 * size + size/2);
    }
}

fn get_light(location: (usize, usize), height: u32, size: usize, j: usize, k: usize) -> f32{
    let actual_location = get_actual_location(location, size);
    let ground_dist = eucl_dist(actual_location, (j, k));
    let tg_a = ground_dist / (height as f32);
    let alpha = tg_a.atan();
    let mut x = j / size;
    let mut y = k / size;
    if x > 2{
        x = 2;
    }
    if y > 2{
        y = 2;
    }
    return alpha.cos() * LIGHT_LUMINOSITY * ALBEDO[3 * x + y];
}

impl LightSource{
    fn init() -> Self{
        let location_ = (0, 0);
        let height_: u32 = 0;
        let shape = (SIZE*3, SIZE*3);
        let light_matrix_ = ndarray::Array2::<f32>::default(shape);
        let is_on_ = false;
        return LightSource { 
            location: location_,
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
                    *col = get_light(self.location, self.height, self.size, j, k);
                }
            });
        }
    }

}

struct Scene{
    scene_array: ndarray::Array2::<f32>,
    scene_image: image::GrayImage,
    size: usize
}

fn decide(sz: usize, j: usize, k:usize) -> f32{
    let mut x = j / sz;
    let mut y = k / sz;
    if x > 2{
        x = 2;
    }
    if y > 2{
        y = 2;
    }
    return COLORS[x * 3+ y] as f32;
}

fn decide_light(orig: f32, lighted: f32) -> f32{
    return orig*lighted;
}

fn clinear_to_srgb(input: f32) -> f32{
    let a: f32 = 0.055;
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

fn srgb_to_clinear(input: usize) -> f32{
    let a: f32 = 0.055;
    let input_01 = (input as f32) / 255.0;
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

fn generate_arr(sz: usize) -> ndarray::Array2::<f32>{
    let shape = (sz*3, sz*3);
    let mut arr = ndarray::Array2::<f32>::default(shape);
    ndarray::Zip::indexed(arr.outer_iter_mut()).par_for_each(|j, mut row| {
        for (k, col) in row.iter_mut().enumerate(){
            *col = decide(sz, j, k);
        }
    });
    return arr
}

fn arr_to_img(arr: &ndarray::Array2::<f32>) -> image::GrayImage{
    let mut img = image::ImageBuffer::new(arr.dim().0 as u32, arr.dim().1 as u32);
    for (r, row) in arr.outer_iter().enumerate()  {
        for (c, col) in row.iter().enumerate(){
            let pixel = (*col).round() as u8;
            let pixel = image::Luma([pixel]);
            img.put_pixel(r as u32, c as u32, pixel);
        }
    }
    return img;  
}

fn generate_arr_and_img(sz: usize)-> (ndarray::Array2::<f32>, image::GrayImage){
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

    fn recount_final_array(&self, light_matrix: &ndarray::Array2::<f32>) -> ndarray::Array2::<f32>{
        let shape = (self.size*3, self.size*3);
        let mut arr = ndarray::Array2::<f32>::default(shape);
        ndarray::Zip::indexed(arr.outer_iter_mut()).par_for_each(|j, mut row| {
            for (k, col) in row.iter_mut().enumerate(){
                *col = decide_light(self.scene_array[[j, k]], light_matrix[[j, k]]);
            }
        });
        return arr;
    }

    fn update(&mut self, ls: &LightSource) -> ndarray::Array2::<f32>{
        let mut new_arr = self.recount_final_array(&ls.light_matrix);
        if !ls.is_on{
            new_arr = self.scene_array.clone();
        }
        self.scene_image = arr_to_img(&new_arr);
        self.scene_image.save("scene.png").unwrap();
        return new_arr;
    }
}

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
                    ui.add(eframe::egui::Checkbox::new(&mut self.light_source.is_on, "Turn the light on"));
                });
            self.img_gui.show(ui);
            if self.light_source.height != 0{
                self.update_();
            }
            if ui.button("Save pic").clicked(){
                let path = "scene_h".to_string() + self.light_source.height.to_string().as_str() + ".png";
                self.scene.scene_image.save(path).unwrap();
            }
            if self.light_source.is_on{
                ui.label("Reverse task soltions:");
                ui.label(format!("height: {}", self.reverse_solution_height));
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
