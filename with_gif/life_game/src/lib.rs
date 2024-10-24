mod vertex;

use std::{error::Error, time::Duration};

use wgpu_helper::{
  context::WgpuContext,
  framework::with_gif::{App, Gif, Render, RenderTarget},
};
use winit::{dpi::PhysicalSize, event::WindowEvent};

// グリッドの縦方向と横方向にそれぞれいくつのセルが存在するか
// 整数値で十分だが、シェーダー側でのキャストが面倒なので最初から浮動小数点値で定義
const GRID_SIZE: f32 = 32.0;

// simulation.wgslの@workgroup_sizeと一致させる必要がある
const WORKGROUP_SIZE: f32 = 8.0;

fn setup() -> (Model, Initial) {
  let model = Model {};

  let initial = Initial {};

  (model, initial)
}

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let (model, initial) = setup();

  let mut app: App<State> =
    App::new("with_gif/life_game", model, initial, None);
  app.run()?;

  Ok(())
}

pub async fn export_gif() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let (model, initial) = setup();

  let mut gif = Gif::<State>::new(512, model, initial, Some(4)).await;
  gif.export("export/with_gif-lige_game.gif", 30, 1).await?;

  Ok(())
}

struct Model {}

struct Initial {}

struct State {}

impl<'a> Render<'a> for State {
  type DrawData = Model;
  type InitialState = Initial;

  async fn new(
    ctx: &WgpuContext<'a>,
    draw_data: &Self::DrawData,
    initial_state: &Self::InitialState,
  ) -> Self {
    todo!()
  }

  fn resize(&mut self, ctx: &WgpuContext, size: Option<PhysicalSize<u32>>) {
    todo!()
  }

  fn process_event(&mut self, event: &WindowEvent) -> bool {
    todo!()
  }

  fn update(&mut self, ctx: &WgpuContext, dt: Duration) {
    todo!()
  }

  fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    target: RenderTarget,
    sample_count: u32,
  ) -> anyhow::Result<Option<wgpu::SurfaceTexture>, wgpu::SurfaceError> {
    todo!()
  }
}
