static COLORS: &'static [u8] =
 &[70, 50, 30, 
  50, 40, 20, 
  40, 20, 10];

static SIZE: usize = 300;
static LIGHT_LUMINOSITY: f32 = 100.0;

fn main(){
    let sim_app = LightSimApp::init(300);
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
    img_gui: egui_extras::RetainedImage
}

fn load_im_egui() -> eframe::epaint::ColorImage{
    let img = image::open("scene.png").unwrap().to_rgba8();
    let rgba = img.as_raw();
    let img_ = eframe::epaint::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], rgba);
    return img_;
}

impl LightSimApp{
    fn init(sz: usize) -> Self{
        let ls = LightSource::init();
        let sc = Scene::init(sz);
        let img_ = load_im_egui();
        return LightSimApp { 
            light_source: ls, 
            scene: sc,
            img_gui: egui_extras::RetainedImage::from_color_image("sceneimg", img_)
        }
    }

    fn update_(&mut self){
        self.light_source.generate_light_matrix();
        self.scene.update(&self.light_source);
        let img_ = load_im_egui();
        self.img_gui = egui_extras::RetainedImage::from_color_image("sceneimg", img_);
    }
}

struct LightSource{
    location: (usize, usize),
    height: u32, //in pixels
    is_on: bool,
    size: usize,
    light_matrix: ndarray::Array2::<f32>
}

fn get_light(location: (usize, usize), height: u32, size: usize, j: usize, k: usize) -> f32{
    if j == size || j == 2 * size || k == size || k == 2 * size{
        return 0.0;
    }
    //actual logic goes here...
    let actual_location = (location.0 * size + size / 2, location.1 * size + size / 2);
    let ground_dist = eucl_dist(actual_location, (j, k));
    let tg_a = ground_dist / (height as f32);
    let alpha = tg_a.atan();
    if alpha == 0.0{
        return LIGHT_LUMINOSITY;
    }
    else{
        return alpha.cos() * alpha.cos() * LIGHT_LUMINOSITY;
    }
    //if j > location.0 * size && k > location.1 * size{
    //    if j < ((location.0+1) * size) && k < (location.1+1) * size{
    //        println!("height");
    //        return height as f32;
    //    }
    //    else{
    //        return 0.0;
    //    }
    //}
    //else{
    //    return 0.0;
    //}
}

impl LightSource{
    fn init() -> Self{
        let location_ = (0, 0);
        let height_: u32 = 0;
        let shape = (SIZE*3 + 2, SIZE*3 + 2);
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
    if j == sz || j == 2 * sz || k == sz || k == 2 * sz{
        return 0.0;
    }
    else{
        let mut x = j / sz;
        let mut y = k / sz;
        if x > 2{
            x = 2;
        }
        if y > 2{
            y = 2;
        }
        return COLORS[x + (3 * y)] as f32;
    } 
}

fn generate_arr(sz: usize) -> ndarray::Array2::<f32>{
    let shape = (sz*3 + 2, sz*3 + 2);
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
            let pixel = *col as u8;
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

    fn update(&mut self, ls: &LightSource){
        let mult: f32 = u8::from(ls.is_on) as f32;
        let new_arr = &self.scene_array + &ls.light_matrix * mult;
        self.scene_image = arr_to_img(&new_arr);
        self.scene_image.save("scene.png").unwrap();
    }
}

impl eframe::App for LightSimApp{
        fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame){
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Light Simulation");
            ui.vertical(|ui|{
                ui.vertical(|ui|{
                    ui.add(eframe::egui::Slider::new(&mut self.light_source.height, 0..=900).text("Light source height"));
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
                    if ui.button("Simulate light").clicked(){
                        self.update_();
                    }
                });
            self.img_gui.show(ui);
            self.update_();
            });
            });
    }    
}
