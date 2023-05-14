use crate::osc;
use anyhow::Result;
use rosc::OscType;
use tokio::sync;

pub static PEN_HANDLER: once_cell::sync::OnceCell<sync::Mutex<PenHandler>> =
    once_cell::sync::OnceCell::new();

#[derive(Debug, Clone, Copy)]
pub struct PenHandler {
    target_state: PenState,
    current_state: PenState,
    speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PenState {
    Idle(f32, f32),
    Drawing(f32, f32),
}

impl Default for PenHandler {
    fn default() -> Self {
        Self {
            target_state: PenState::default(),
            current_state: PenState::default(),
            speed: 1000.0,
        }
    }
}

impl Default for PenState {
    fn default() -> Self {
        Self::Idle(0.0, 0.0)
    }
}

impl PenState {
    pub fn idle_from_pos(pos: eframe::egui::Pos2) -> Self {
        Self::Idle(pos.x, pos.y)
    }

    pub fn drawing_from_pos(pos: eframe::egui::Pos2) -> Self {
        Self::Drawing(pos.x, pos.y)
    }

    pub fn enable_drawing(&mut self) {
        if let Self::Idle(x, y) = self {
            *self = Self::Drawing(*x, *y);
        }
    }
}

impl PenHandler {
    pub fn new(current_state: PenState, speed: Option<f32>) -> Self {
        let target_state = current_state;
        let speed = speed.unwrap_or(0.0);

        Self {
            target_state,
            current_state,
            speed,
        }
    }

    pub fn init(current_state: Option<PenState>) -> Result<()> {
        match PEN_HANDLER.set(sync::Mutex::new(Self::default())) {
            Ok(_) => Ok(()),
            Err(e) => anyhow::bail!("failed to init pen handler: {:?}", e),
        }
    }
}

impl PenHandler {
    const MOV_PREFIX: &str = "/move_";
    const ON_MOVING: &str = "/on_moving";

    const RIGHT: &str = "right";
    const LEFT: &str = "left";
    const UP: &str = "up";
    const DOWN: &str = "down";

    pub fn new_handler(&self, target_state: PenState) -> PenHandler {
        let mut new_handler = (*self).clone();
        new_handler.set_target_state(target_state);

        new_handler
    }

    fn set_target_state(&mut self, state: PenState) {
        self.target_state = state;
    }

    fn set_current_state(&mut self, state: PenState) {
        self.current_state = state;
    }

    fn draw(&self) -> Result<()> {
        crate::osc::send_packet("/on_drawing", vec![OscType::Bool(true)])?;
        Ok(())
    }

    fn calc_delta(&self) -> (f32, f32) {
        let targegt_position = match self.target_state {
            PenState::Idle(x, y) => (x, y),
            PenState::Drawing(x, y) => (x, y),
        };

        let current_position = match self.current_state {
            PenState::Idle(x, y) => (x, y),
            PenState::Drawing(x, y) => (x, y),
        };

        (
            targegt_position.0 - current_position.0,
            targegt_position.1 - current_position.1,
        )
    }

    async fn _mov_to(&self, dst: &str, time: f32) -> Result<()> {
        let speed = self.speed;

        osc::send_packet(
            format!("{}{dst}", Self::MOV_PREFIX).as_str(),
            vec![OscType::Float(speed)],
        )?;
        log::info!("Is moving to {}", dst);
        osc::send_packet(Self::ON_MOVING, vec![OscType::Bool(true)])?;

        tokio::time::sleep(tokio::time::Duration::from_secs_f32(time)).await;

        osc::send_packet(Self::ON_MOVING, vec![OscType::Bool(false)])?;
        log::info!("Has moved to {}", dst);

        Ok(())
    }

    async fn mov(&self) -> Result<()> {
        let delta = self.calc_delta();

        let time = (delta.0.powi(2) + delta.1.powi(2)).sqrt() / self.speed;

        log::info!("Is changing the state into: {:?}", self.target_state);
        log::info!("Time: {:?}", time);

        if delta.0 > 0.0 {
            self._mov_to(Self::RIGHT, time).await?;
        }

        if delta.0 < 0.0 {
            self._mov_to(Self::LEFT, time).await?;
        }

        if delta.1 > 0.0 {
            self._mov_to(Self::UP, time).await?;
        }

        if delta.1 < 0.0 {
            self._mov_to(Self::DOWN, time).await?;
        }

        log::info!("Has changed the state into: {:?}", self.target_state);

        Ok(())
    }

    pub async fn eval(&self) -> PenHandler {
        let mut newer_handler = (*self).clone();

        if self.target_state == newer_handler.current_state {
            return newer_handler;
        }

        match self.mov().await {
            Ok(_) => {
                newer_handler.set_current_state(self.target_state);

                newer_handler
            }
            Err(e) => {
                log::error!("Failed to move with: {}", e);
                newer_handler
            }
        }
    }
}

/*

    fn mov(&mut self) -> Result<()> {
        let delta = self.calc_delta();

        let time = (delta.0.powi(2) + delta.1.powi(2)).sqrt() / self.speed;

        log::info!("Is moving to target position: {:?}", self.target_state);
        log::info!("Time: {:?}", time);

        loop {
            if let Some(task) = self.tasks.pop_front() {
                match task {
                    MoveState::Right(time) => {
                        self._mov_to(Self::RIGHT, time)?;
                    }
                    MoveState::Left(time) => {
                        self._mov_to(Self::LEFT, time)?;
                    }
                    MoveState::Up(time) => {
                        self._mov_to(Self::UP, time)?;
                    }
                    MoveState::Down(time) => {
                        self._mov_to(Self::DOWN, time)?;
                    }
                }
            } else {
                break;
            }
        }

        log::info!("Has moved to target position: {:?}", self.target_state);

        Ok(())
    }
*/
