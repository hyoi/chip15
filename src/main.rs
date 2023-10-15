//external crates
use bevy::
{   prelude::*,
    log::LogPlugin,
    core_pipeline::clear_color::ClearColorConfig,
    window::WindowMode,
    asset::LoadState,
    render::camera,
    diagnostic::DiagnosticsStore,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::mouse,
    utils::HashMap,
    utils::HashSet,
    sprite::Anchor,
    sprite::MaterialMesh2dBundle,
};
use once_cell::sync::Lazy;
use counted_array::counted_array;
use rand::prelude::*;
use regex::Regex;

//standard library
use std::ops::{ Range, Add, AddAssign };
use std::f32::consts::{ PI, TAU, FRAC_PI_2 };

//internal submodules
mod public;
use public::*;

mod load_assets;
mod init_app;
mod play_game;

////////////////////////////////////////////////////////////////////////////////

//メイン関数
fn main()
{   //アプリの生成
    let mut app = App::new();

    //メインウィンドウの設定
    let primary_window = MAIN_WINDOW.clone();
    let log_level = if misc::DEBUG() { LOG_LEVEL_DEV } else { LOG_LEVEL_REL };
    let filter = log_level.into();
    app
    .insert_resource( Msaa::Sample4 ) //アンチエイリアス
    .add_plugins
    (   DefaultPlugins
        .set( WindowPlugin { primary_window, ..default() } ) //メインウィンドウ
        .set( ImagePlugin::default_nearest() ) //ピクセルパーフェクト
        .set( LogPlugin { filter, ..default() } ) //ロギング
    )
    .add_systems
    (   Update,
        (   (   bevy::window::close_on_esc, //[ESC]で終了
                misc::toggle_window_mode,   //フルスクリーン切換
            )
            .run_if( not( misc::WASM ) ),
        )
    )
    ;

    //カメラとライトを作る
    app
    .add_systems
    (   Startup,
        (   misc::spawn_2d_camera, //2D camera
            misc::spawn_3d_camera, //3D camera
            misc::spawn_3d_light,  //3D light

            //テスト用：オブジェクト表示
            (   debug::spawn_2d_sprites, //2D表示テスト
                debug::spawn_3d_objects, //3D表示テスト
            )
            .run_if( misc::DEBUG )
            .run_if( not( resource_exists::<AfterInitAppTo<MyState>>() ) )
        )
    )
    .add_systems
    (   Update,
        //テスト用：3Dカメラを極座標上で動かす
        (   (   debug::catch_input_keyboard, //キー入力
                debug::catch_input_mouse,    //マウス
                debug::catch_input_gamepad,  //ゲームパッド
            ),
            debug::move_orbit_camera::<Camera3d> //カメラの移動
                .run_if( any_with_component::<misc::AppDefault3dCamera>() ),
        )
        .chain() //実行順の固定
        // .run_if( misc::DEBUG )
    )
    ;

    //メイン処理
    app
    .add_state::<MyState>() //Stateを初期化する。enumの#[default]で初期値指定
    .add_plugins( load_assets::Schedule ) //assetsの事前ロード
    .add_plugins( init_app::Schedule )    //ゲーム枠・FPSの表示等、事前処理
    .add_plugins( play_game::Schedule )   //ゲームロジック
    ;

    //アプリの実行
    app.run();
}

////////////////////////////////////////////////////////////////////////////////

//End of code.