struct MainState {
    world: World,
    schedule: Schedule,
    mouse_down: bool,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        // Create the world
        let mut world = World::default();

        // Create the celestial
        let planet = EarthLikeBuilder::new().build();
        world.spawn(planet);

        // Create the camera
        let _screen_size = ctx.gfx.drawable_size();
        let camera = Camera::new(ScreenSize(Vec2::new(_screen_size.0, _screen_size.1)));
        world.insert_resource(camera);

        // Create the camera window
        let camera_window = CameraWindow::new(&ctx);
        world.insert_resource(camera_window);

        // Create the cursor tooltip
        let cursor_tooltip = CursorTooltip::new(&ctx, &camera);
        world.insert_resource(cursor_tooltip);

        // Create the element picker
        let element_picker = ElementPicker::new(&mut ctx);
        world.insert_resource(element_picker);

        // Add the context to the world
        world.insert_resource(ctx);

        // Create the brush
        let brush = Brush::default();
        world.insert_resource(brush);

        // Create the global clock
        let current_time = GlobalClock::default();
        world.insert_resource(current_time);

        // Create the schedule
        let mut schedule = Schedule::default();

        // Return the world
        Ok(MainState {
            world,
            schedule,
            mouse_down: false,
        })
    }
}
