use super::*;

//internal submodules
mod map;
mod player;
mod minimap;

mod spawn_methods;
use spawn_methods::*;

////////////////////////////////////////////////////////////////////////////////

//プラグインの設定
pub struct Schedule;
impl Plugin for Schedule
{   fn build( &self, app: &mut App )
    {   app
        //InitAppの実行後にGameStartへ遷移させる
        .insert_resource( AfterInitAppTo ( MyState::GameStart ) )

        //Resourceの登録
        .init_resource::<map::Map>()       //マップ情報
        .init_resource::<player::Player>() //プレイヤー情報
        .init_resource::<OrbitCamera>()    //極座標カメラ情報

        //ゲームプレイ前の処理
        .add_systems
        (   OnEnter ( MyState::GameStart ),
            (   //Playerカメラ(Fpp&Tpp)を作るので、
                //AppDefaultな3Dカメラを削除する(※1)
                misc::despawn::<misc::AppDefault3dCamera>,

                //ミニマップ用2Dカメラとスプライト
                minimap::spawn_minimap,

                misc::change_state::<MainLoop>, //無条件遷移
            )
        )

        //ステージの前処理
        .add_systems
        (   OnEnter ( MyState::MainLoop ),
            (   map::make_new_data,     //新しいMapデータを作る
                map::spawn_entity,      //Mapを3D表示する
                player::spawn_entity,   //playerと3Dカメラのspawn
                minimap::setup_minimap, //ミニマップの初期表示

                //Playerカメラ(Fpp&Tpp)がない場合(ほぼデバッグ時)、
                //AppDefaultな3Dカメラ(があれば)をmapのstartへ向ける(※1)
                look_at_map_start::<misc::AppDefault3dCamera>
                    .run_if( any_with_component::<misc::AppDefault3dCamera>() ),
            )
            .chain() //実行順の固定
        )

        //メインループ
        .add_systems
        (   Update,
            (   //テスト用：三人称視点カメラ有効化
                (   switch_fpp_and_tpp, //[Space]キーでカメラを切り替える
                    debug::move_orbit_camera::<player::TppCamera>, //カメラの移動
                ),
                // .run_if( misc::DEBUG ),
 
                (   //Playerを操作する
                    (   player::catch_input_keyboard, //キー入力
                        // player::catch_input_mouse,    //マウス
                        // player::catch_input_gamepad,  //ゲームパッド
                    ),
                    (   player::rotate_player, //playerの向きを変える
                        player::move_player,   //playerを移動する
                    ),
                    (   minimap::turn_player, //ミニマップ上のプレイヤーの旋回
                        minimap::move_camera, //ミニマップ用2Dカメラの位置更新
                    ),
                )
                .chain()
            )
            .run_if( in_state( MyState::MainLoop ) )
        );
    }
}

////////////////////////////////////////////////////////////////////////////////

//テスト用：カメラをmap.startへ向ける
fn look_at_map_start<T: Component>
(   mut que_camera: Query<(&mut Transform, &Camera), With<T>>,
    mut orbit_camera: ResMut<OrbitCamera>,
    map: Res<map::Map>,
)
{   let Ok ( ( mut transform, camera ) ) = que_camera.get_single_mut() else { return };
    if ! camera.is_active { return }

    //OrbitCameraを初期化する
    *orbit_camera = OrbitCamera
    {   look_at: map.start.to_3dxz(), //mapのstart位置
        is_active: camera.is_active,  //カメラの現時点の状態を保存
        ..default()
    };

    //カメラの位置と向きを更新する
    let origin = orbit_camera.look_at;
    let vec3   = orbit_camera.orbit.to_vec3() + origin;
    *transform = Transform::from_translation( vec3 ).looking_at( origin, Vec3::Y );
}

////////////////////////////////////////////////////////////////////////////////

//テスト用：一人称視点カメラ⇔三人称視点カメラ切替
fn switch_fpp_and_tpp
(   mut que_cameras: Query<&mut Camera, Or<( &player::FppCamera, &player::TppCamera )>>, 
    mut orbit_camera: ResMut<OrbitCamera>,
    inkey: Res<Input<KeyCode>>,
)
{   if ! inkey.just_pressed( KeyCode::Space ) { return } //[Space]キー

    //FppとTppのカメラの状態を反転させる
    que_cameras.for_each_mut( | mut camera | camera.is_active = ! camera.is_active );

    //極座標カメラ(Tpp)用のResourceを書き換える
    orbit_camera.is_active = ! orbit_camera.is_active; //反転
}

////////////////////////////////////////////////////////////////////////////////

//End of code.