use super::*;

////////////////////////////////////////////////////////////////////////////////

//.run_if( condition )用
pub const DEBUG: fn() -> bool = || cfg!( debug_assertions );
pub const WASM : fn() -> bool = || cfg!( target_arch = "wasm32" );

////////////////////////////////////////////////////////////////////////////////

//2D cameraをspawnする
pub fn spawn_2d_camera( mut cmds: Commands )
{   //2Dカメラを第四象限に移動する
    //左上隅が(0,0)、X軸はプラス方向へ伸び、Y軸はマイナス方向へ下がる
    let translation = Vec3::X     * SCREEN_PIXELS_WIDTH  * 0.5
                    + Vec3::NEG_Y * SCREEN_PIXELS_HEIGHT * 0.5;

    cmds.spawn( Camera2dBundle::default() )
    .insert( Camera { order: ORDER_CAMERA2D_DEFAULT, ..default() } )
    .insert( Camera2d { clear_color: CAMERA2D_BGCOLOR } )
    .insert( Transform::from_translation( translation) )
    ;
}

//デフォルトの3D CameraのComponent
#[derive( Component )]
pub struct AppDefault3dCamera;

//3D cameraをspawnする
pub fn spawn_3d_camera( mut cmds: Commands )
{   let _id = 
    cmds.spawn( ( Camera3dBundle:: default(), AppDefault3dCamera ) )
    .insert( Camera { order: ORDER_CAMERA3D_DEFAULT, ..default() } )
    .insert( Camera3d { clear_color: CAMERA3D_BGCOLOR, ..default() } )
    .id()
    ;

    //debug時にcameraのtransformをセットする
    //（ここでセットしないと期待した表示にならなかった）
    #[cfg( debug_assertions )]
    cmds.entity( _id )
    .insert
    (   Transform::from_translation
        (   Orbit
            {   r    : ORBIT_CAMERA_INIT_R,
                theta: ORBIT_CAMERA_INIT_THETA,
                phi  : ORBIT_CAMERA_INIT_PHI,
            }
            .to_vec3()
        )
        .looking_at( Vec3::ZERO, Vec3::Y )
    );
}

//3D lightをspawnする
pub fn spawn_3d_light( mut cmds: Commands )
{   let illuminance = LIGHT3D_BRIGHTNESS;
    let shadows_enabled = true;
    let light = DirectionalLight { illuminance, shadows_enabled, ..default() };
    let transform = Transform::from_translation( LIGHT3D_TRANSLATION );

    cmds.spawn( DirectionalLightBundle::default() )
    .insert( light )
    .insert( transform.looking_at( Vec3::ZERO, Vec3::Y ) )
    ;
}

////////////////////////////////////////////////////////////////////////////////

//3D Cameraにviewport(表示エリア)をセットする
pub fn set_viewport
(   mut q_camera: Query<&mut Camera, With<Camera3d>>,
)
{   q_camera.for_each_mut
    (   | mut camera |
        camera.viewport = Some
        (   camera::Viewport
            {   physical_position: SCREEN_FRAME.viewport.origin.as_uvec2(),
                physical_size    : SCREEN_FRAME.viewport.size.as_uvec2(),
                ..default()
            }
        )
    );
}

////////////////////////////////////////////////////////////////////////////////

//ウィンドウとフルスクリーンの切換(トグル動作)
pub fn toggle_window_mode
(   mut q_window: Query<&mut Window>,
    keys: Res<Input<KeyCode>>,
    gpdbtn: Res<Input<GamepadButton>>,
    gamepads: Res<Gamepads>,
)
{   let Ok( mut window ) = q_window.get_single_mut() else { return };

    //[Alt]＋[Enter]の状態
    let is_key_pressed =
        ( keys.pressed( KeyCode::AltRight ) || keys.pressed( KeyCode::AltLeft ) )
            && keys.just_pressed( KeyCode::Return );

    //ゲームパッドは抜き挿しでIDが変わるので.iter()で回す
    let button_type = GamepadButtonType::Select; //ps4[SHARE]
    let mut is_gpdbtn_pressed = false;
    for gamepad in gamepads.iter()
    {   if gpdbtn.just_pressed( GamepadButton { gamepad, button_type } )
        {   is_gpdbtn_pressed = true;
            break;
        }
    }

    //入力がないなら
    if ! is_key_pressed && ! is_gpdbtn_pressed { return }

    //ウィンドウとフルスクリーンを切り替える
    window.mode = match window.mode
    {   WindowMode::Windowed => WindowMode::SizedFullscreen,
        _                    => WindowMode::Windowed,
    };
}

////////////////////////////////////////////////////////////////////////////////

//QueryしたEnityを再帰的に削除する
pub fn despawn<T: Component>
(   q_entity: Query<Entity, With<T>>,
    mut cmds: Commands,
)
{   q_entity.for_each( | id | cmds.entity( id ).despawn_recursive() );
}

////////////////////////////////////////////////////////////////////////////////

//Stateの無条件遷移
pub fn change_state<T: Send + Sync + Default + GotoState>
(   state: Local<T>,
    mut next_state: ResMut<NextState<MyState>>
)
{   next_state.set( state.next() );
}

//Stateの無条件遷移（Resourceで遷移先指定）
pub fn change_state_with_res<T: Resource + GotoState>
(   o_state: Option<Res<T>>,
    mut next_state: ResMut<NextState<MyState>>
)
{   let Some ( state ) = o_state else { warn!( "No exists State." ); return };

    next_state.set( state.next() );
}

////////////////////////////////////////////////////////////////////////////////

//End of code.