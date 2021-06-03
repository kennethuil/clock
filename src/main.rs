use std::{marker::PhantomData};
use std::fmt::Debug;
use chrono::{DateTime, DurationRound, Local, Timelike, Duration};
use druid::{Affine, Color, Rect, RenderContext};
use druid::{AppLauncher, Data, Event, TimerToken, Widget, WidgetExt, WindowDesc, widget::Controller};

#[derive(Clone, Debug)]
struct ClockTime<T: Timelike + Clone + Debug + 'static>(T);

impl<T: Timelike + Clone + Debug + 'static> Data for ClockTime<T> {
    fn same(&self, other: &Self) -> bool {
        // one second resolution for now
        self.0.hour() == other.0.hour() &&
        self.0.minute() == other.0.minute() &&
        self.0.second() == other.0.second()
    }
}
struct AnalogClock<T: Timelike + Clone + Debug +'static> {
    phantom: PhantomData<T>
}

impl<T: Timelike + Clone + Debug + 'static> AnalogClock<T> {
    fn new() -> AnalogClock<T> {
        AnalogClock {
            phantom: PhantomData{}
        }
    }
}

impl<T: Timelike + Clone + Debug + 'static> Widget<ClockTime<T>> for AnalogClock<T> {
    fn event(&mut self, _ctx: &mut druid::EventCtx, _event: &druid::Event, _data: &mut ClockTime<T>, _env: &druid::Env) {
    }

    fn lifecycle(&mut self, _ctx: &mut druid::LifeCycleCtx, _event: &druid::LifeCycle, _data: &ClockTime<T>, _env: &druid::Env) {
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, _old_data: &ClockTime<T>, _data: &ClockTime<T>, _env: &druid::Env) {
        ctx.request_paint();
    }

    fn layout(&mut self, _ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, _data: &ClockTime<T>, _env: &druid::Env) -> druid::Size {
        bc.constrain_aspect_ratio(1.0, 400.0)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &ClockTime<T>, _env: &druid::Env) {
        let size = ctx.size();
        let ClockTime(time) = data;

        let max_length = if size.width > size.height {
            size.height
        } else {
            size.width
        } / 2.0;
        let center = (size.width / 2.0, size.height / 2.0);

        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate(center));

            let mark_color = Color::rgb8(0xc0, 0x00, 0x40);
            for i in 0..60 {
                ctx.with_save(|ctx| {
                    ctx.transform(Affine::rotate((std::f64::consts::PI / 30.0) * (i as f64)));

                    let mark = if i % 5 == 0 {
                        Rect::from_points((-3.0, -max_length), (2.0, 20.0-max_length))
                    } else {
                        Rect::from_points((-1.0, 17.0-max_length), (1.0, 20.0-max_length))
                    };
                    //let fill_color = Color::rgb((i as f64) / 60.0, 0.0, (60-i) as f64 / 60.0);
                    ctx.fill(mark, &mark_color);
                })
            }

            // Hour hand
            ctx.with_save(|ctx| {
                ctx.transform(Affine::rotate(std::f64::consts::PI * 2.0 * (time.num_seconds_from_midnight() as f64) * 2.0 / 86400.0));
                let hand = Rect::from_points((-6.0, 0.0), (6.0, (20.0 - max_length) / 2.0));
                let fill_color = Color::GREEN;
                ctx.fill(hand, &fill_color);
            });

            // Minute hand
            ctx.with_save(|ctx| {
                ctx.transform(Affine::rotate(std::f64::consts::PI * 2.0 * (time.num_seconds_from_midnight() as f64) / 3600.0));
                let hand = Rect::from_points((-3.0, 0.0), (2.0, 20.0 - max_length));
                let fill_color = Color::GREEN;
                ctx.fill(hand, &fill_color);
            });

            /*
            // Second hand
            ctx.with_save(|ctx| {
                ctx.transform(Affine::rotate(std::f64::consts::PI * 2.0 * (time.second() as f64) / 60.0));
                let hand = Rect::from_points((-1.0, 0.0), (1.0, 20.0 - max_length));
                let fill_color = Color::GRAY;
                ctx.fill(hand, &fill_color);
            });
            */
        })
    }
}

// now() is only implemented on Utc and Local (timezones), and we're mainly interested in local time
struct LocalClockController {
    timer_token: TimerToken
}

impl LocalClockController {
    fn new() -> LocalClockController {
        LocalClockController {
            timer_token: TimerToken::INVALID
        }
    }
    // Wake up time should be the next whole second
    fn get_timer_interval() -> std::time::Duration {
        let now = Local::now();
        let nanos_left = 1_000_000_000 - now.nanosecond();
        Duration::nanoseconds(nanos_left.into()).to_std().expect("Impossible OutOfRangeError")
    }
}

impl<W: Widget<ClockTime<DateTime<Local>>>> Controller<ClockTime<DateTime<Local>>, W> for LocalClockController {
    fn event(&mut self, child: &mut W, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut ClockTime<DateTime<Local>>, env: &druid::Env) {
        match event {
            Event::WindowConnected => {
                self.timer_token = ctx.request_timer(LocalClockController::get_timer_interval());
            }
            Event::Timer(id) => {
                if *id == self.timer_token {
                    self.timer_token = ctx.request_timer(LocalClockController::get_timer_interval());
                }
            }
            _ => ()
        }

        // TODO: Rounding should be handled in the clock widget instead, but the Timelike interface
        // doesn't support that directly
        *data = ClockTime(Local::now().duration_round(Duration::seconds(1)).expect("Nanos out of range"));
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &ClockTime<DateTime<Local>>,
        env: &druid::Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, child: &mut W, ctx: &mut druid::UpdateCtx, old_data: &ClockTime<DateTime<Local>>, data: &ClockTime<DateTime<Local>>, env: &druid::Env) {
        child.update(ctx, old_data, data, env)
    }
}

fn build_simple_analog_clock() -> impl Widget<ClockTime<DateTime<Local>>> {
    AnalogClock::new()
        .controller(LocalClockController::new())
}

fn main() {
    let initial_state = ClockTime(Local::now());
    AppLauncher::with_window(
        WindowDesc::new(build_simple_analog_clock)
            .title("Clock")
            .window_size((200.0, 200.0)))
        .launch(initial_state).expect("Failed to launch window");
}
