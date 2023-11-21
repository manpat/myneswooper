use toybox::prelude::*;
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};


pub struct SoundSystem {
	cmd_tx: SyncSender<ProviderCmd>,
}

impl SoundSystem {
	pub fn start(audio: &mut audio::System) -> anyhow::Result<SoundSystem> {
		let (cmd_tx, cmd_rx) = sync_channel(4);

		let provider = Provider {
			cmd_rx,
			sounds: Vec::new(),
			dt: 0.0
		};

		audio.set_provider(provider)?;

		Ok(SoundSystem{cmd_tx})
	}

	pub fn play(&self, sound: Sound) {
		self.cmd_tx.send(ProviderCmd::Play(sound)).unwrap();
	}
}


pub enum Sound {
	Plik,
	Bong,
	Thup,
	Unthup,
	Tada,
}



enum ProviderCmd {
	Play(Sound)
}




struct Provider {
	cmd_rx: Receiver<ProviderCmd>,

	sounds: Vec<SoundState>,
	dt: f64,
}

impl audio::Provider for Provider {
	fn on_configuration_changed(&mut self, cfg: audio::Configuration) {
		println!("[provider] Configuration changed {cfg:?}");
		self.dt = (cfg.sample_rate as f64).recip();
	}

	fn fill_buffer(&mut self, buffer: &mut [f32]) {
		for cmd in self.cmd_rx.try_iter() {
			match cmd {
				ProviderCmd::Play(sound) => {
					self.sounds.push(SoundState {
						sound,
						phase: 0.0,
						env: 0.0,
					});
				}
			}
		}

		buffer.fill(0.0);

		for sound in self.sounds.iter_mut() {
			sound.fill(buffer, self.dt);
		}

		self.sounds.retain(|s| !s.is_finished());
	}
}


struct SoundState {
	sound: Sound,
	phase: f64,
	env: f64,
}

impl SoundState {
	fn fill(&mut self, buffer: &mut [f32], dt: f64) {
		for [l, r] in buffer.array_chunks_mut() {
			let sample = match self.sound {
				Sound::Plik => generate_plik_sample(&mut self.phase, &mut self.env, dt),
				Sound::Bong => generate_bong_sample(&mut self.phase, &mut self.env, dt),
				Sound::Thup => generate_thup_sample(&mut self.phase, &mut self.env, dt),
				Sound::Unthup => generate_unthup_sample(&mut self.phase, &mut self.env, dt),
				Sound::Tada => generate_tada_sample(&mut self.phase, &mut self.env, dt),
			};

			let sample = sample * 0.3;

			*l += sample;
			*r += sample;
		}
	}

	fn is_finished(&self) -> bool {
		self.env > 1.0
	}
}


use std::f64::consts::TAU;

fn generate_plik_sample(phase: &mut f64, env_phase: &mut f64, dt: f64) -> f32 {
	let env = gen_env(*env_phase);
	let osc = (*phase * TAU).sin() as f32;

	*phase = (*phase + dt * 600.0 * (1.0 - env_phase.powi(2)*0.7)).fract();
	*env_phase += dt * 12.0;

	osc * env * 0.8
}

fn generate_bong_sample(phase: &mut f64, env_phase: &mut f64, dt: f64) -> f32 {
	let env = gen_env(*env_phase);
	let osc = (*phase * TAU * 1.0).sin() as f32;
	let osc2 = (*phase * TAU * (2.0 - *env_phase*0.3)).sin() as f32;

	*phase += dt * (120.0 + (*env_phase * 36.0).sin() * 5.0 - *env_phase*40.0);
	*env_phase += dt / 2.2;

	(osc + osc2) * env * 0.6
}

fn generate_thup_sample(phase: &mut f64, env_phase: &mut f64, dt: f64) -> f32 {
	let env = gen_env(*env_phase);
	let osc = (*phase * TAU).sin() as f32;

	*phase = (*phase + dt * 210.0 * 2.0 / 3.0 * (1.0 - env_phase.powi(2) * 0.6)).fract();
	*env_phase += dt / 0.05;

	osc * env
}

fn generate_unthup_sample(phase: &mut f64, env_phase: &mut f64, dt: f64) -> f32 {
	let env = gen_env(*env_phase);
	let osc = (*phase * TAU).sin() as f32;

	*phase = (*phase + dt * 210.0 * (1.0 + env_phase.powi(2) * 0.9)).fract();
	*env_phase += dt / 0.1;

	osc * env
}

fn generate_tada_sample(phase: &mut f64, env_phase: &mut f64, dt: f64) -> f32 {
	let env = gen_env(*env_phase);
	let env2 = gen_env(*env_phase - 0.1);
	let osc = (*phase * TAU).sin() as f32;
	let osc2 = (*phase * TAU * 3.0 / 2.0).sin() as f32;

	*phase = (*phase + dt * 800.0) % 6.0;
	*env_phase += dt / 1.0;

	(osc * env + osc2 * env2) / 2.0
}

fn gen_env(phase: f64) -> f32 {
	if phase < 0.0 || phase > 1.0 {
		return 0.0
	}

	if phase < 0.02 {
		1.0 - (1.0 - phase/0.02).powi(2).clamp(0.0, 1.0) as f32

	} else {
		(1.0 - phase/0.98).powi(3).clamp(0.0, 1.0) as f32
	}
}