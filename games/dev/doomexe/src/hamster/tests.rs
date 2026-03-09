#[cfg(test)]
mod tests {
    use crate::hamster::components::*;
    use crate::hamster::HamsterPlugin;
    use bevy::prelude::*;

    #[test]
    fn test_expression_cycling() {
        let mut expr = Expression::Neutral;

        expr = expr.next();
        assert_eq!(expr, Expression::Happy);

        expr = expr.next();
        assert_eq!(expr, Expression::Angry);

        expr = expr.next();
        assert_eq!(expr, Expression::Neutral);
    }

    #[test]
    fn test_corruption_effect_updates() {
        let mut effect = CorruptionEffect::default();

        // Test 0 corruption
        effect.update_from_level(0.0);
        assert_eq!(effect.level, 0.0);
        assert_eq!(effect.palette_index, 0);
        assert_eq!(effect.jitter_amplitude, 0.0);
        assert_eq!(effect.chromatic_aberration, 0.0);

        // Test 50 corruption
        effect.update_from_level(50.0);
        assert_eq!(effect.level, 50.0);
        assert_eq!(effect.palette_index, 2); // 50 / 25 = 2
        assert_eq!(effect.jitter_amplitude, 0.25); // 50 / 200 = 0.25
        assert_eq!(effect.chromatic_aberration, 1.0); // (50 / 100) * 2.0 = 1.0

        // Test 100 corruption
        effect.update_from_level(100.0);
        assert_eq!(effect.level, 100.0);
        assert_eq!(effect.palette_index, 4); // 100 / 25 = 4
        assert_eq!(effect.jitter_amplitude, 0.5);
        assert_eq!(effect.chromatic_aberration, 2.0);
    }

    #[test]
    fn test_expression_sprite_handles() {
        let h_neutral = Handle::default();
        let h_happy = Handle::default();
        let h_angry = Handle::default();

        let sprites = ExpressionSprite {
            neutral: h_neutral.clone(),
            happy: h_happy.clone(),
            angry: h_angry.clone(),
        };

        // We can't easily compare handles for equality in unit tests without a world,
        // but we can check if the method returns a reference.
        // Actually Handle<T> implements PartialEq and Eq if T does?
        // Bevy Handles are just IDs. They should be comparable.

        assert_eq!(sprites.handle_for(Expression::Neutral), &h_neutral);
        assert_eq!(sprites.handle_for(Expression::Happy), &h_happy);
        assert_eq!(sprites.handle_for(Expression::Angry), &h_angry);
    }

    #[test]
    fn test_plugin_initialization() {
        let mut app = App::new();
        // Use MinimalPlugins for headless testing, plus AssetPlugin for the AssetServer
        // We also need ImagePlugin to register the Image asset type used by sprites
        // And InputPlugin for keyboard input resources
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .add_plugins(ImagePlugin::default_nearest())
            .add_plugins(bevy::input::InputPlugin)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<crate::state::GameState>()
            .add_plugins(HamsterPlugin);

        // Run one frame to ensure startup systems and one cycle of update systems run
        app.update();

        // Verify that the hamster root was spawned
        let world = app.world_mut();
        let mut root_query = world.query::<&CharacterRoot>();
        assert_eq!(
            root_query.iter(world).count(),
            1,
            "Should spawn exactly one hamster root"
        );
    }
}
