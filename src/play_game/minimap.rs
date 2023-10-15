use super::*;

////////////////////////////////////////////////////////////////////////////////

//ミニマップの各セルのComponent
#[derive( Component, Debug )]
pub struct MiniMapCell ( pub IVec2 );

//ミニマップ上の三角形のComponent
#[derive( Component )]
pub struct MinimapPlayer;

//ミニマップ用2DカメラのComponent
#[derive( Component )]
pub struct MinimapCamera;

//ミニマップのスプライトの情報
const COLOR_SPRITE_MINIMAP_CELL_BASE: Color = GROUND_PLANE_COLOR;
const COLOR_SPRITE_MINIMAP_CELL_WALL: Color = WALL_CUBE_COLOR;
const COLOR_SPRITE_MINIMAP_PLAYER   : Color = Color::YELLOW;
const RADIUS_SPRITE_MINIMAP_PLAYER  : f32 = PIXELS_PER_GRID * 0.3; //正多角形の外接円の半径

////////////////////////////////////////////////////////////////////////////////

//ミニマップ用のスプライトと2Dカメラをspawnする
pub fn spawn_minimap
(   mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_color: ResMut<Assets<ColorMaterial>>,
    asset_svr: Res<AssetServer>, //for debug
)
{   //ミニマップにスプライトを敷き詰める
    let scale = if misc::DEBUG() { 0.9 } else { 1.0 };
    let custom_size = Some ( SIZE_GRID * scale );
    let color = COLOR_SPRITE_MINIMAP_CELL_BASE;
    let x_grids_range = 0..MAP_GRIDS_WIDTH  + SCREEN_FRAME.minimap.size.x;
    let y_grids_range = 0..MAP_GRIDS_HEIGHT + SCREEN_FRAME.minimap.size.y;
    let x_grids_adjuster = IVec2::X * SCREEN_GRIDS_WIDTH; //X軸で１画面分ずらす

    for y in y_grids_range
    {   for x in x_grids_range.clone()
        {   //スプライトをspawn
            let grid = IVec2::new( x, y );
            let work = grid + x_grids_adjuster;
            let translation = work.to_screen_pixel().extend( DEPTH_SPRITE_MINIMAP );
            cmds.spawn( ( SpriteBundle::default(), MiniMapCell ( grid ) ) )
            .insert( Sprite { custom_size, color, ..default() } )
            .insert( Transform::from_translation( translation ) )
            .with_children //for debug
            (   | cmds |
                {   if misc::DEBUG()
                    {   //debug用文字列
                        let value = format!( "{:02}\n{:02}", x, y ).to_string();
                        let style = TextStyle
                        {   font     : asset_svr.load( ASSETS_FONT_PRESSSTART2P_REGULAR ),
                            font_size: PIXELS_PER_GRID * 0.25,
                            color    : Color::DARK_GREEN,
                        };
                        let sections  = vec![ TextSection { value, style } ];
                        let alignment = TextAlignment::Center;

                        cmds.spawn( Text2dBundle::default() )
                        .insert( Text { sections, alignment, ..default() } )
                        .insert( Transform::from_translation( Vec3::Z ) )
                        ;
                    }
                }
            );
        }
    }

    //プレイヤーの三角形をspawnする
    let radius = RADIUS_SPRITE_MINIMAP_PLAYER;
    let triangle = MaterialMesh2dBundle
    {   mesh: meshes.add( shape::RegularPolygon::new( radius, 3 ).into() ).into(),
        material: materials_color.add( ColorMaterial::from( COLOR_SPRITE_MINIMAP_PLAYER ) ),
        visibility: Visibility::Hidden, //ちらっと見えるので無効化しておく
        ..default()
    };
    let grid = SCREEN_FRAME.minimap.zero + SCREEN_FRAME.minimap.size / 2;
    let translation = grid.to_screen_pixel().extend( DEPTH_SPRITE_MINIMAP_PLAYER );

    cmds.spawn( ( triangle, MinimapPlayer ) )
    .insert( Transform::from_translation( translation ) )
    ;

    //ミニマップ用2Dカメラをspawnする
    let zero = SCREEN_FRAME.minimap.zero.as_vec2() * PIXELS_PER_GRID;
    let size = SCREEN_FRAME.minimap.size.as_vec2() * PIXELS_PER_GRID;
    let viewport = Some
    (   camera::Viewport
        {   physical_position: zero.as_uvec2(),
            physical_size    : size.as_uvec2(),
            ..default()
        }
    );
    let order = ORDER_CAMERA2D_MINIMAP;
    let is_active = false; //ちらっと見えるので無効化しておく
    cmds.spawn( ( Camera2dBundle::default(), MinimapCamera ) )
    .insert( Camera { viewport, order, is_active, ..default() } )
    .insert( Camera2d { clear_color: CAMERA2D_BGCOLOR } )
    ;
}

////////////////////////////////////////////////////////////////////////////////

//ミニマップの初期表示
#[allow(clippy::type_complexity)]
pub fn setup_minimap
(   mut qry_minimap_sprite: Query<( &mut Sprite, &MiniMapCell )>,
    mut param_set: ParamSet
    <(  Query<( &mut Transform, &mut Visibility ), With<MinimapPlayer> >,
        Query<( &mut Transform, &mut Camera ), With<MinimapCamera> >,
    )>,
    opt_map: Option<Res<map::Map>>,
    opt_player: Option<Res<player::Player>>,
)
{   let Some ( map ) = opt_map else { return };
    let Some ( player ) = opt_player else { return };

    //ミニマップのスプライト更新の準備
    let mut sprite_hash = HashMap::new();
    qry_minimap_sprite.for_each_mut( | ( s, m ) | { sprite_hash.insert( m.0, s ); } );
    let x_range = 0..MAP_GRIDS_WIDTH  + SCREEN_FRAME.minimap.size.x;
    let y_range = 0..MAP_GRIDS_HEIGHT + SCREEN_FRAME.minimap.size.y;
    let adjuster = SCREEN_FRAME.minimap.size / 2; //奇数／２の場合の端数は切り捨て

    //ミニマップのスプライトの表示を更新する
    for y in y_range
    {   for x in x_range.clone()
        {   let grid = IVec2::new( x, y );
            let Some ( sprite ) = sprite_hash.get_mut( &grid ) else { continue };

            //スプライトの色を変更
            sprite.color = match map.is_wall( grid - adjuster )
            {   true  => COLOR_SPRITE_MINIMAP_CELL_WALL,
                false => COLOR_SPRITE_MINIMAP_CELL_BASE,
            };
        }
    }

    //プレイヤーの三角形の向きをセットする
    let mut qry_player = param_set.p0();
    if let Ok ( ( mut transform, mut visibility ) ) = qry_player.get_single_mut()
    {   transform.rotation = player.direction.to_quat_z();
        *visibility = Visibility::Visible;
    }

    //ミニマップ用2Dカメラをmapのstart位置にセットする
    let mut qry_camera = param_set.p1();
    if let Ok ( ( mut transform, mut camera ) ) = qry_camera.get_single_mut()
    {   transform.translation = map.start.to_minimap_center();
        camera.is_active = true; //カメラの有効化
    }
}

////////////////////////////////////////////////////////////////////////////////

//プレイヤーの左右旋回に合わせ三角形を回転する
pub fn turn_player
(   mut qry_minimap_player: Query<&mut Transform, With<MinimapPlayer>>,
    opt_player: Option<Res<player::Player>>,
    time: Res<Time>,
    mut flag_ongoing: Local<bool>, //default false 停止
)
{   let Ok ( mut triangle ) = qry_minimap_player.get_single_mut() else { return };
    let Some ( player ) = opt_player else { return };

    //左右旋回でないなら
    if ! ( player.is_turn() || *flag_ongoing ) { return }

    if ! player.is_turn() //暗黙に *flag_ongoing は true
    {   //プレイヤーの三角形の向きをピッタリにする
        triangle.rotation = player.direction.to_quat_z();

        *flag_ongoing = false; //行動終了
    }
    else
    {   //プレイヤーの三角形を左右旋回（中間アニメーション）
        let time_delta = time.delta().as_secs_f32(); //微小時間
        let radian = UNIT_TURN * time_delta * PLAYER_TURN_COEF;
        let delta = Quat::from_rotation_z( radian );

        match player.in_action
        {   player::InAction::TurnRight => triangle.rotation *= delta.inverse(),
            player::InAction::TurnLeft  => triangle.rotation *= delta,
            _ => (),
        }

        *flag_ongoing = true; //行動中
    }
}

////////////////////////////////////////////////////////////////////////////////

//プレイヤー前進後退に合わせてミニマップを逆方向へずらすため、カメラを移動する
pub fn move_camera
(   mut qry_minimap_camera: Query<&mut Transform, With<MinimapCamera>>,
    opt_player: Option<Res<player::Player>>,
    time: Res<Time>,
    mut flag_ongoing: Local<bool>, //default false 停止
)
{   let Ok ( mut camera ) = qry_minimap_camera.get_single_mut() else { return };
    let Some ( player ) = opt_player else { return };

    //前進後退でないなら
    if ! ( player.is_move() || *flag_ongoing ) { return }

    if ! player.is_move() //暗黙に *flag_ongoing は true
    {   //ミニマップ用2Dカメラの位置をピッタリにする
        camera.translation = player.position.to_minimap_center();

        *flag_ongoing = false; //行動終了
    }
    else
    {   //ミニマップ用2Dカメラを前進後退（中間アニメーション）
        let vector = match player.direction
        {   News::North => Vec3::Y,
            News::East  => Vec3::X,
            News::West  => Vec3::NEG_X,
            News::South => Vec3::NEG_Y,
        };
        let time_delta = time.delta().as_secs_f32(); //微小時間
        let delta = PIXELS_PER_GRID * time_delta * PLAYER_MOVE_COEF * vector;

        match player.in_action
        {   player::InAction::Forward  => camera.translation += delta,
            player::InAction::Backward => camera.translation -= delta,
            _ => (),
        }

        *flag_ongoing = true; //行動中
    }
}

////////////////////////////////////////////////////////////////////////////////

trait MinimapTrait
{   //ミニマップ用2Dカメラの座標を計算する
    fn to_minimap_center( &self ) -> Vec3;
}

impl MinimapTrait for IVec2
{   //ミニマップ用2Dカメラの座標を計算する
    fn to_minimap_center( &self ) -> Vec3
    {   let adjuster = SCREEN_FRAME.minimap.size.as_vec2() * PIXELS_PER_GRID * 0.5;
        let origin = Vec2::X * SCREEN_PIXELS_WIDTH;
        let neg_y = Vec2::X + Vec2::NEG_Y; //Y軸を符号反転させるための係数
        let center = origin + self.as_vec2() * PIXELS_PER_GRID * neg_y;

        ( center + adjuster * neg_y ).extend( 0.0 ) //2DカメラのZ軸はダミー？
    }
}

////////////////////////////////////////////////////////////////////////////////

//End of code.