use super::*;

////////////////////////////////////////////////////////////////////////////////

//PlayerのResource
#[derive( Resource, Default )]
pub struct Player
{   pub position : IVec2,    //位置
    pub direction: News,     //向き
    pub in_action: InAction, //行動の種類
}

#[derive( Clone, Copy, Default, PartialEq )]
pub enum InAction
{   #[default] Stop,
    TurnRight, TurnLeft, //左右回転
    Forward, Backward,   //前進後退
}

impl Player
{   //停止か？
    pub fn is_stop( &self ) -> bool
    {   self.in_action == InAction::Stop
    }
    //左右旋回か？
    pub fn is_turn( &self ) -> bool
    {   self.in_action == InAction::TurnRight || self.in_action == InAction::TurnLeft
    }
    //前進後退か？
    pub fn is_move( &self ) -> bool
    {   self.in_action == InAction::Forward || self.in_action == InAction::Backward
    }
}

// プレイヤーの構造
// PlayerEntity       ：親 (clear box)
//  ├─FigureHead      ：中間親 (clear box)
//  │　├─(FigureParts)：キャラクターの姿
//  |　└─FppCamera    ：一人称視点カメラ
//  └─TppCamera       ：三人称視点カメラ

//プレイヤー(親)のComponent
#[derive( Component, Clone, Copy, Default )]
pub struct PlayerEntity;

//プレイヤーの姿(中間親)のComponent
#[derive( Component )]
pub struct FigureHead;

//カメラのComponent
#[derive( Component )]
pub struct FppCamera; //一人称視点カメラ
#[derive( Component )]
pub struct TppCamera; //三人称視点カメラ

////////////////////////////////////////////////////////////////////////////////

//Playerの3Dオブジェクトをspawnする
#[allow(clippy::too_many_arguments)]
pub fn spawn_entity
(   qry_entity: Query<Entity, With<PlayerEntity>>,
    mut player: ResMut<Player>,
    mut map: ResMut<map::Map>,
    mut orbit_camera: ResMut<OrbitCamera>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_standard: ResMut<Assets<StandardMaterial>>,
)
{   //既存のPlayerがあれば削除する
    qry_entity.for_each( | id | cmds.entity( id ).despawn_recursive() );

    //Playerの設定
    let sides = map.get_sides_space( map.start );
    let side = sides[ map.rng.gen_range( 0..sides.len() ) ];
    let direction = side;
    *player = Player { position: map.start, direction, ..default() };

    let player_position  = player.position.to_3dxz();
    let player_direction = player.direction.to_quat_y();

    //一人称視点カメラの設定
    let is_active = true;

    //三人称視点カメラ(極座標カメラ)の設定
    *orbit_camera = OrbitCamera { is_active: ! is_active, ..default() };
    let orbit_position = orbit_camera.orbit.to_vec3();

    //透明な箱をspawnし、それを親にして中に子をspawnする
    cmds.spawn( ( PbrBundle::default(), PlayerEntity ) )
    .insert( materials_standard.add( Color::NONE.into() ) ) //透明
    .insert( Transform::from_translation( player_position ) ) //位置
    .with_children
    (   | mut cmds |
        {   //透明な箱をspawnし、それを親にして中に子をspawnする
            cmds.spawn( ( PbrBundle::default(), FigureHead ) )
            .insert( materials_standard.add( Color::NONE.into() ) ) //透明
            .insert( Transform::from_rotation( player_direction ) ) //向き
            .with_children
            (   | mut cmds |
                {   //Playerの姿をspawnする
                    cmds.spawn_player_figure( &mut meshes, &mut materials_standard );

                    //一人称視点カメラをspawnする
                    //一人称視点カメラはPlayerの中心(Vec3::ZERO)にあり正面を向いている
                    let target = Vec3::NEG_Z; //正面をNEG_Z(News::North)に固定する
                    let position = Vec3::Z * 0.478; //画角を稼ぐため背面方向へカメラを少し引く
                    cmds.spawn_player_camera3d( FppCamera, is_active, position, target );
                }
            );

            //三人称視点カメラをspawnする（極座標カメラ）
            let is_active = orbit_camera.is_active; //一人称視点カメラと反対の状態にする
            let target = Vec3::ZERO; //注視点はPlayer自身なのでVec3::ZERO
            cmds.spawn_player_camera3d( TppCamera, is_active, orbit_position, target );
        }
    );
}

////////////////////////////////////////////////////////////////////////////////

//キー入力によって自機の位置と向きを更新する
pub fn catch_input_keyboard
(   mut player: ResMut<Player>,
    map: Res<map::Map>,
    orbit_camera: Res<OrbitCamera>,
    inkey: Res<Input<KeyCode>>,
)
{   //三人称視点カメラがアクティブなら、入力を受け付けない
    if orbit_camera.is_active { return }

    //Playerが停止していない場合、入力を受け付けない
    if ! player.is_stop() { return }

    //自機の位置と向きを更新する
    for keycode in inkey.get_just_pressed()
    {   match keycode
        {   KeyCode::Right =>
            {   player.direction = player.direction.turn_right();
                player.in_action = InAction::TurnRight;
            }
            KeyCode::Left =>
            {   player.direction = player.direction.turn_left();
                player.in_action = InAction::TurnLeft;
            }
            KeyCode::Up =>
            {   let front = player.position + player.direction;
                if ! map.is_noentry( front )
                {   player.position = front;
                    player.in_action = InAction::Forward;
                }
            }
            KeyCode::Down =>
            {   let back = player.position + player.direction.back();
                if ! map.is_noentry( back )
                {   player.position = back;
                    player.in_action = InAction::Backward;
                }
            }
            _ => (),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

//プレイヤーを左右旋回する
pub fn rotate_player
(   mut qry_figure: Query<&mut Transform, With<FigureHead>>,
    mut player: ResMut<Player>,
    time: Res<Time>,
    mut sum_radian: Local<f32>, //default 0.0
)
{   let Ok ( mut figure ) = qry_figure.get_single_mut() else { return };

    //左右旋回でないなら
    if ! player.is_turn() { return }

    //微小時間の回転角度
    let time_delta = time.delta().as_secs_f32(); //微小時間
    let radian = UNIT_TURN * time_delta * PLAYER_TURN_COEF;
    *sum_radian += radian; //累積を保存する

    //累積が1単位を超えたら
    if *sum_radian >= UNIT_TURN
    {   //向きをピッタリにする
        figure.rotation = player.direction.to_quat_y();

        //情報更新する
        player.in_action = InAction::Stop;
        *sum_radian = 0.0;
    }
    else
    {   //左右旋回する（中間アニメーション）
        let delta = Quat::from_rotation_y( radian );
        match player.in_action
        {   InAction::TurnRight => figure.rotation *= delta.inverse(),
            InAction::TurnLeft  => figure.rotation *= delta,
            _ => (),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

//プレイヤーを前進後退させる
pub fn move_player
(   mut qry_player: Query<&mut Transform, With<PlayerEntity>>,
    mut player: ResMut<Player>,
    time: Res<Time>,
    mut sum_distance: Local<f32>, //default 0.0
)
{   let Ok ( mut transform ) = qry_player.get_single_mut() else { return };

    //前進後退でないなら
    if ! player.is_move() { return }

    //微小時間の移動距離
    let time_delta = time.delta().as_secs_f32();
    let delta = UNIT_MOVE * time_delta * PLAYER_MOVE_COEF;
    *sum_distance += delta; //累積を保存する

    //累積が1単位を超えたら
    if *sum_distance >= UNIT_MOVE
    {   //位置をピッタリにする
        *transform = Transform::from_translation( player.position.to_3dxz() );

        //情報更新する
        player.in_action = InAction::Stop;
        *sum_distance = 0.0;
    }
    else
    {   //前進後退する（中間アニメーション）
        let delta_vec3 = delta * match player.direction
        {   News::North => Vec3::NEG_Z,
            News::East  => Vec3::X,
            News::West  => Vec3::NEG_X,
            News::South => Vec3::Z,
        };
        match player.in_action
        {   InAction::Forward  => transform.translation += delta_vec3,
            InAction::Backward => transform.translation -= delta_vec3,
            _ => (),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

//End of code.