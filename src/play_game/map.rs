use super::*;

//internal submodules
mod maze_a;
mod maze_b;

////////////////////////////////////////////////////////////////////////////////

//MapのResource
#[derive( Resource )]
pub struct Map
{   pub rng: rand::prelude::StdRng, //専用乱数発生器
    matrix: Vec<Vec<Flag>>,         //map
    pub start: IVec2,               //スタート位置
}

//マスの情報
#[derive( Clone )]
struct Flag ( u128 );

//Map::default()の定義
impl Default for Map
{   fn default() -> Self
    {   let seed_dev = 1234567890;
        let seed_rel = || rand::thread_rng().gen::<u64>();
        let seed = if misc::DEBUG() { seed_dev } else { seed_rel() };

        let cell = Flag ( BIT_CELL_UNDEF );
        let column = vec![ cell  ; MAP_GRIDS_HEIGHT as usize ];
        let matrix = vec![ column; MAP_GRIDS_WIDTH  as usize ];

        Self
        {   rng  : StdRng::seed_from_u64( seed ),
            matrix,
            start: IVec2::default(),
        }
    }
}

//マス目の状態を表すビット(フラグは128個まで)
const BIT_CELL_UNDEF      : u128 = 0b000000; //未定義
const BIT_CELL_SPACE      : u128 = 0b000001; //地形：空地
const BIT_CELL_WALL       : u128 = 0b000010; //地形：壁
const BIT_FLAG_FOOTPRINTS : u128 = 0b000100; //フラグ：足跡
const BIT_FLAG_NOENTRY    : u128 = 0b001000; //フラグ：進入禁止
const BIT_FLAG_DEADEND    : u128 = 0b010000; //フラグ：袋小路
const BIT_FLAG_LOCKEDCHEST: u128 = 0b100000; //フラグ：袋小路

//Mapの全Entityの親になるEntityに印をつけるComponent
#[derive( Component )]
pub struct MapZeroEntity;

////////////////////////////////////////////////////////////////////////////////

//Mapのメソッド
impl Map
{   //ユーティリティ
    fn is_inside( &self, cell: IVec2 ) -> bool
    {   MAP_GRIDS_X_RANGE.contains( &cell.x ) &&
        MAP_GRIDS_Y_RANGE.contains( &cell.y )
    }
    fn matrix_mut( &mut self, IVec2 { x, y }: IVec2 ) -> &mut Flag
    {   &mut self.matrix[ x as usize ][ y as usize ]
    }
    fn matrix( &self, IVec2 { x, y }: IVec2 ) -> &Flag
    {   &self.matrix[ x as usize ][ y as usize ]
    }

    //全体を埋める
    fn fill_walls( &mut self )
    {   self.matrix.iter_mut().for_each
        (   |column| column.fill( Flag ( BIT_CELL_WALL ) )
        );
    }

    //指定の位置の地形を書き換える（フラグはクリアされる）
    fn set_space( &mut self, cell: IVec2 )
    {   if ! self.is_inside( cell ) { return }
        *self.matrix_mut( cell ) = Flag ( BIT_CELL_SPACE );
    }

    //指定の位置にフラグを付加する
    fn add_flag_footprints( &mut self, cell: IVec2 )
    {   if ! self.is_inside( cell ) { return }
        self.matrix_mut( cell ).0 |= BIT_FLAG_FOOTPRINTS;
    }
    fn add_flag_noentry( &mut self, cell: IVec2 )
    {   if ! self.is_inside( cell ) { return }
        self.matrix_mut( cell ).0 |= BIT_FLAG_NOENTRY;
    }
    fn add_flag_deadend( &mut self, cell: IVec2 )
    {   if ! self.is_inside( cell ) { return }
        self.matrix_mut( cell ).0 |= BIT_FLAG_DEADEND;
    }
    fn add_flag_lockedchest( &mut self, cell: IVec2 )
    {   if ! self.is_inside( cell ) { return }
        self.matrix_mut( cell ).0 |= BIT_FLAG_LOCKEDCHEST;
    }

    //袋小路を探してフラグを付加する
    fn search_deadend( &mut self )
    {   for x in MAP_GRIDS_X_RANGE
        {   for y in MAP_GRIDS_Y_RANGE
            {   let cell = IVec2::new( x, y );

                //空地じゃないなら（壁なら）
                if ! self.is_space( cell )
                {   self.add_flag_noentry( cell ); //親友禁止の目印
                    continue;
                }
                self.set_space( cell ); //map作成時のflagを消去するためにsetし直す

                //袋小路の目印
                if self.get_sides_space( cell ).len() == 1
                {   self.add_flag_deadend( cell );
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

//Mapのpubメソッド
impl Map
{   //cellの四方を調べて空地がある方角のVecを返す
    pub fn get_sides_space( &self, cell: IVec2 ) -> Vec< News >
    {   //四方の空地を探し記録する
        let mut sides = Vec::with_capacity( 4 );
        for news in NEWS
        {   if self.is_space( cell + news ) { sides.push( news ) }
        }

        sides //空地がある方角のVec
    }

    //指定の位置の地形・フラグを判定する
    pub fn is_wall( &self, cell: IVec2 ) -> bool
    {   if ! self.is_inside( cell ) { return true } //範囲外は壁にする
        self.matrix( cell ).0 & BIT_CELL_WALL != 0
    }
    pub fn is_space( &self, cell: IVec2 ) -> bool
    {   if ! self.is_inside( cell ) { return false } //範囲外に空地はない
        self.matrix( cell ).0 & BIT_CELL_SPACE != 0
    }
    pub fn is_footprints( &self, cell: IVec2 ) -> bool
    {   if ! self.is_inside( cell ) { return false } //範囲外に空地はない(＝足跡もない)
        self.matrix( cell ).0 & BIT_FLAG_FOOTPRINTS != 0
    }
    pub fn is_noentry( &self, cell: IVec2 ) -> bool
    {   if ! self.is_inside( cell ) { return false } //範囲外に空地はない(＝進入禁止もない)
        self.matrix( cell ).0 & BIT_FLAG_NOENTRY != 0
    }
    pub fn is_deadend( &self, cell: IVec2 ) -> bool
    {   if ! self.is_inside( cell ) { return false } //範囲外に空地はない(＝袋小路もない)
        self.matrix( cell ).0 & BIT_FLAG_DEADEND != 0
    }
    pub fn is_lockedchest( &self, cell: IVec2 ) -> bool
    {   if ! self.is_inside( cell ) { return false } //範囲外に空地はない(＝袋小路もない)
        self.matrix( cell ).0 & BIT_FLAG_LOCKEDCHEST != 0
    }
}

////////////////////////////////////////////////////////////////////////////////

//新しいMapデータを作る
pub fn make_new_data( mut map: ResMut<Map> )
{   //初期化する
    map.fill_walls();
    map.start = if misc::DEBUG()
    {   //DEBUG: 開始位置をマップ中央に固定
        IVec2::new( MAP_GRIDS_WIDTH / 2, MAP_GRIDS_HEIGHT / 2 )
    }
    else
    {   //迷路生成関数に任せる（開始位置を指定しない）
        IVec2::NEG_ONE 
    };

    //迷路を作る
    match map.rng.gen_range( 0..3 )
    {   0 => { map.build_maze_a_1().build_maze_b(); },
        1 => { map.build_maze_a_2().build_maze_b(); },
        2 => { map.build_maze_b(); },
        _ => (),
    }

    //迷路の構造を解析してフラグを付加する＜仮＞
    map.search_deadend();

    //イベントオブジェクトのフラグを追加する＜仮＞
    for x in MAP_GRIDS_X_RANGE
    {   for y in MAP_GRIDS_Y_RANGE
        {   let cell = IVec2::new( x, y );

            if map.is_deadend( cell )
            && map.rng.gen_bool( 1.0 / 3.0 )
            && cell != map.start
            {   map.add_flag_noentry( cell ); //進入禁止の目印
                map.add_flag_lockedchest( cell ); //＜仮＞
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

//迷路の3Dオブジェクトをspawnする
pub fn spawn_entity
(   q_entity: Query<Entity, With<MapZeroEntity>>,
    map: Res<Map>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
)
{   //既存のEntityがあれば削除する
    q_entity.for_each( | id | cmds.entity( id ).despawn_recursive() );

    //壁のサイズ、原点の壁のテクスチャ、他の壁のテクスチャ、地面のテクスチャ
    let size = WALL_CUBE_SIZE * if misc::DEBUG() { 0.95 } else { 1.0 };
    let texture_wall_zero =
    (   if misc::DEBUG() { WALL_CUBE_COLOR_ZERO } else { WALL_CUBE_COLOR }
    ) .into();
    let texture_wall_normal: StandardMaterial = WALL_CUBE_COLOR.into();
    let texture_ground = GROUND_PLANE_COLOR.into();

    //迷路をspawnする
    cmds.spawn( ( PbrBundle::default(), MapZeroEntity ) ) //Cube(親)
    .insert( meshes.add( shape::Cube::new( size ).into() ) )
    .insert( Transform::from_translation( Vec3::ZERO ) ) //原点
    .insert( materials.add( texture_wall_zero ) )
    .with_children
    (   | mut cmds |
        {   //子は、親からの相対位置にspawnされる(XZ平面)
            for x in MAP_GRIDS_X_RANGE
            {    for y in MAP_GRIDS_Y_RANGE
                {   //原点は親なのでスキップ
                    if x == 0 && y == 0 { continue }

                    //3D空間の座標
                    let cell = IVec2::new( x, y );
                    let vec3 = cell.to_3dxz();

                    //壁
                    if map.is_wall( cell )
                    {   cmds.spawn( PbrBundle::default() )
                        .insert( meshes.add( shape::Cube::new( size ).into() ) )
                        .insert( Transform::from_translation( vec3 ) )
                        .insert( materials.add( texture_wall_normal.clone() ) )
                        ;
                    }

                    //宝箱
                    if map.is_lockedchest( cell ) //＜仮＞
                    {   let quat = map.get_sides_space( cell )[ 0 ].to_quat_y();
                        cmds.spawn_locked_chest( vec3, quat, &mut meshes, &mut materials );
                    }
                }
            }

            //地面も相対位置でspawnする
            let long_side = MAP_GRIDS_WIDTH.max( MAP_GRIDS_HEIGHT ) as f32;
            let half = long_side / 2.0;
            let position = Vec3::new( half, 0.0, half ) - Vec3::ONE / 2.0;
            cmds.spawn( PbrBundle::default() )
            .insert( meshes.add( shape::Plane::from_size( long_side ).into() ) )
            .insert( Transform::from_translation( position ) )
            .insert( materials.add( texture_ground ) )
            ;
        }
    );
}

////////////////////////////////////////////////////////////////////////////////

//End of code.