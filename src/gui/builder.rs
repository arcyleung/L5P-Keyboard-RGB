use super::enums::Message;
use super::keyboard_manager;
use super::{effect_browser_tile, enums::BaseColor, enums::Effects, keyboard_color_tiles, options_tile};
use fltk::app::Sender;
use fltk::{
	app,
	enums::{Event, Font},
	group::Pack,
	input::IntInput,
	prelude::*,
	window::Window,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

const WIDTH: i32 = 900;
const HEIGHT: i32 = 450;

pub fn start_ui(mut manager: keyboard_manager::KeyboardManager, tx: Sender<Message>, stop_signal: Arc<AtomicBool>) -> fltk::window::Window {
	//UI
	let mut win = Window::default().with_size(WIDTH, HEIGHT).with_label("Legion Keyboard RGB Control");
	let mut color_picker_pack = Pack::new(0, 0, 540, 360, "");
	let mut keyboard_color_tiles = create_keyboard_color_tiles(&tx, stop_signal.clone());

	color_picker_pack.add(&keyboard_color_tiles.zones.left.exterior_tile);
	color_picker_pack.add(&keyboard_color_tiles.zones.center_left.exterior_tile);
	color_picker_pack.add(&keyboard_color_tiles.zones.center_right.exterior_tile);
	color_picker_pack.add(&keyboard_color_tiles.zones.right.exterior_tile);
	color_picker_pack.add(&keyboard_color_tiles.master.exterior_tile);
	color_picker_pack.end();

	let effects_list: Vec<&str> = vec![
		"Static",
		"Breath",
		"Smooth",
		"LeftWave",
		"RightWave",
		"Lightning",
		"AmbientLight",
		"SmoothLeftWave",
		"SmoothRightWave",
		"LeftSwipe",
		"RightSwipe",
	];
	let effect_browser_tile = effect_browser_tile::EffectBrowserTile::create(&effects_list);
	let mut effect_browser = effect_browser_tile.effect_browser;

	let options_tile = options_tile::OptionsTile::create();
	let mut speed_choice = options_tile.speed_choice;
	let mut brightness_choice = options_tile.brightness_choice;

	win.end();
	win.make_resizable(false);
	win.show();

	// Theming
	app::background(51, 51, 51);
	app::set_visible_focus(false);
	app::set_font(Font::HelveticaBold);

	//Begin app logic
	//Speed
	speed_choice.set_callback({
		let tx = tx.clone();
		move |choice| {
			if let Some(value) = choice.choice() {
				let speed = value.parse::<u8>().unwrap();
				if (1..=4).contains(&speed) {
					tx.send(Message::UpdateSpeed { speed });
				}
			}
		}
	});

	//Brightness
	brightness_choice.set_callback({
		let tx = tx.clone();
		move |choice| {
			if let Some(value) = choice.choice() {
				let brightness = value.parse::<u8>().unwrap();
				if (1..=2).contains(&brightness) {
					tx.send(Message::UpdateBrightness { brightness });
				}
			}
		}
	});

	// Effect choice
	effect_browser.set_callback({
		let stop_signal = stop_signal.clone();
		move |browser| match browser.value() {
			0 => {
				browser.select(0);
			}

			1 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::Static });
			}
			2 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::Breath });
			}
			3 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::Smooth });
			}
			4 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::LeftWave });
			}
			5 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::RightWave });
			}
			6 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::Lightning });
			}
			7 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::AmbientLight });
			}
			8 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::SmoothLeftWave });
			}
			9 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::SmoothRightWave });
			}
			10 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::LeftSwipe });
			}
			11 => {
				stop_signal.store(true, Ordering::Relaxed);
				tx.send(Message::UpdateEffect { effect: Effects::RightSwipe });
			}
			_ => {}
		}
	});
	thread::spawn(move || {
		manager.start(&mut keyboard_color_tiles, &mut speed_choice, &stop_signal);
	});
	win
}

fn create_keyboard_color_tiles(tx: &Sender<Message>, stop_signal: Arc<AtomicBool>) -> keyboard_color_tiles::KeyboardColorTiles {
	fn add_zone_tile_handle(control_tile: &mut keyboard_color_tiles::ColorTile, tx: &Sender<Message>, zone_index: u8, stop_signal: Arc<AtomicBool>) {
		fn add_input_handle(input: &mut IntInput, color: BaseColor, tx: Sender<Message>, zone_index: u8, stop_signal: Arc<AtomicBool>) {
			let triplet_index = zone_index * 3;
			let color_index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			let index = triplet_index + color_index;
			input.handle({
				move |input, event| match event {
					Event::KeyUp => {
						match input.value().parse::<f32>() {
							Ok(value) => {
								input.set_value(&value.to_string());
								if value > 255.0 {
									input.set_value("255");
								}
								if !stop_signal.load(Ordering::Relaxed) {
									tx.send(Message::UpdateValue {
										index,
										value: input.value().parse().unwrap(),
									});
								}
							}
							Err(_) => {
								input.set_value("0");
							}
						}
						true
					}
					_ => false,
				}
			});
		}
		//Button
		control_tile.toggle_button.handle({
			let mut control_tile = control_tile.clone();
			let tx = tx.clone();
			move |button, event| match event {
				Event::Released => {
					if button.is_toggled() {
						tx.send(Message::UpdateZone { zone_index, value: [0.0; 3] });
						control_tile.red_input.deactivate();
						control_tile.green_input.deactivate();
						control_tile.blue_input.deactivate();
					} else {
						tx.send(Message::UpdateZone {
							zone_index,
							value: [
								control_tile.red_input.value().parse::<f32>().unwrap(),
								control_tile.green_input.value().parse::<f32>().unwrap(),
								control_tile.blue_input.value().parse::<f32>().unwrap(),
							],
						});
						control_tile.red_input.activate();
						control_tile.green_input.activate();
						control_tile.blue_input.activate();
					}
					true
				}
				_ => false,
			}
		});
		//Red
		add_input_handle(&mut control_tile.red_input, BaseColor::Red, tx.clone(), zone_index, stop_signal.clone());
		//Green
		add_input_handle(&mut control_tile.green_input, BaseColor::Green, tx.clone(), zone_index, stop_signal.clone());
		//Blue
		add_input_handle(&mut control_tile.blue_input, BaseColor::Blue, tx.clone(), zone_index, stop_signal);
	}

	fn add_master_tile_handle(keyboard_color_tiles: &mut keyboard_color_tiles::KeyboardColorTiles, tx: &Sender<Message>, stop_signal: Arc<AtomicBool>) {
		fn add_master_input_handle(input: &mut IntInput, color: BaseColor, tx: Sender<Message>, keyboard_color_tiles: keyboard_color_tiles::KeyboardColorTiles, stop_signal: Arc<AtomicBool>) {
			let index = match color {
				BaseColor::Red => 0,
				BaseColor::Green => 1,
				BaseColor::Blue => 2,
			};
			input.handle({
				let mut keyboard_color_tiles = keyboard_color_tiles;
				move |input, event| match event {
					Event::KeyUp => {
						if let Ok(value) = input.value().parse::<f32>() {
							input.set_value(&value.to_string());
							if value > 255.0 {
								input.set_value("255");
							}
							if !stop_signal.load(Ordering::Relaxed) {
								tx.send(Message::UpdateRGB {
									index,
									value: input.value().parse().unwrap(),
								});
								keyboard_color_tiles.zones.change_color_value(color, input.value().parse().unwrap());
							}
						} else {
							input.set_value("0");
							keyboard_color_tiles.zones.change_color_value(color, 0.0);
						}
						true
					}
					_ => false,
				}
			});
		}
		let mut master_tile = keyboard_color_tiles.master.clone();
		//Button
		master_tile.toggle_button.handle({
			let mut keyboard_color_tiles = keyboard_color_tiles.clone();
			let mut master_tile = master_tile.clone();
			let tx = tx.clone();
			move |button, event| match event {
				Event::Released => {
					if button.is_toggled() {
						tx.send(Message::UpdateAllValues { value: [255.0; 12] });
						master_tile.red_input.deactivate();
						master_tile.green_input.deactivate();
						master_tile.blue_input.deactivate();
						keyboard_color_tiles.zones.deactivate();
					} else {
						let zones = &keyboard_color_tiles.zones;
						tx.send(Message::UpdateAllValues {
							value: [
								zones.left.red_input.value().parse::<f32>().unwrap(),
								zones.left.green_input.value().parse::<f32>().unwrap(),
								zones.left.blue_input.value().parse::<f32>().unwrap(),
								zones.center_left.red_input.value().parse::<f32>().unwrap(),
								zones.center_left.green_input.value().parse::<f32>().unwrap(),
								zones.center_left.blue_input.value().parse::<f32>().unwrap(),
								zones.center_right.red_input.value().parse::<f32>().unwrap(),
								zones.center_right.green_input.value().parse::<f32>().unwrap(),
								zones.center_right.blue_input.value().parse::<f32>().unwrap(),
								zones.right.red_input.value().parse::<f32>().unwrap(),
								zones.right.green_input.value().parse::<f32>().unwrap(),
								zones.right.blue_input.value().parse::<f32>().unwrap(),
							],
						});
						master_tile.red_input.activate();
						master_tile.green_input.activate();
						master_tile.blue_input.activate();
						keyboard_color_tiles.zones.activate();
					}
					true
				}
				_ => false,
			}
		});
		//Red
		add_master_input_handle(&mut master_tile.red_input, BaseColor::Red, tx.clone(), keyboard_color_tiles.clone(), stop_signal.clone());
		//Green
		add_master_input_handle(&mut master_tile.green_input, BaseColor::Green, tx.clone(), keyboard_color_tiles.clone(), stop_signal.clone());
		//Blue
		add_master_input_handle(&mut master_tile.blue_input, BaseColor::Blue, tx.clone(), keyboard_color_tiles.clone(), stop_signal);
	}

	let mut keyboard_color_tiles = keyboard_color_tiles::KeyboardColorTiles {
		master: (keyboard_color_tiles::ColorTile::create(true)),
		zones: keyboard_color_tiles::ZoneColorTiles::create(),
	};

	add_zone_tile_handle(&mut keyboard_color_tiles.zones.left, tx, 0, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.center_left, tx, 1, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.center_right, tx, 2, stop_signal.clone());
	add_zone_tile_handle(&mut keyboard_color_tiles.zones.right, tx, 3, stop_signal.clone());
	add_master_tile_handle(&mut keyboard_color_tiles.clone(), tx, stop_signal);

	keyboard_color_tiles
}
