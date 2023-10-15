use super::*;

////////////////////////////////////////////////////////////////////////////////

//glamの型にメソッドを追加する準備
pub trait GridToPixel
{   fn to_screen_pixel( &self ) -> Vec2;
    fn to_3dxz( &self ) -> Vec3;
}

//glamの型にメソッドを追加する
impl GridToPixel for IVec2
{   //平面座標(IVec2)からスクリーン用の座標(Vec2)を算出する
    fn to_screen_pixel( &self ) -> Vec2
    {   let mut vec2 = ( self.as_vec2() + 0.5 ) * PIXELS_PER_GRID;
        vec2.y *= -1.0; //2Dカメラが第四象限にあるのでY軸は符号が反転する

        vec2
    }

    //平面座標(IVec2)から3D直交座標(Vec3)へ変換する
    fn to_3dxz( &self ) -> Vec3
    {   let x = self.x as f32;
        let y = 0.0; //xz平面上
        let z = self.y as f32;
        Vec3::new( x, y, z )
    }

}

////////////////////////////////////////////////////////////////////////////////

//極座標の型
#[derive( Clone, Copy )]
pub struct Orbit
{   pub r    : f32, //極座標のr（注目点からカメラまでの距離）
    pub theta: f32, //極座標のΘ（注目点から見たカメラの垂直角度）
    pub phi  : f32, //極座標のφ（注目点から見たカメラの水平角度）
}

impl Orbit
{   //極座標から直交座標へ変換する
    #[allow(clippy::wrong_self_convention)]
    pub fn to_vec3( &self ) -> Vec3
    {   let x = self.r * self.theta.sin() * self.phi.sin();
        let y = self.r * self.theta.cos() * -1.0;
        let z = self.r * self.theta.sin() * self.phi.cos();

        Vec3::new( x, y, z )
    }
}

////////////////////////////////////////////////////////////////////////////////

//極座標カメラのResource
#[derive( Resource, Clone, Copy )]
pub struct OrbitCamera
{   pub orbit: Orbit,    //カメラ自身の極座標
    pub look_at: Vec3,   //カメラの注視点の直交座標
    pub is_active: bool, //カメラがアクティブか否か
}

impl Default for OrbitCamera
{   fn default() -> Self
    {   Self
        {   orbit: Orbit
            {   r    : ORBIT_CAMERA_INIT_R,
                theta: ORBIT_CAMERA_INIT_THETA,
                phi  : ORBIT_CAMERA_INIT_PHI,
            },
            look_at: Vec3::ZERO,
            is_active: false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

//ゲームの状態
#[derive( Clone, Copy, Eq, PartialEq, Hash, Debug, Default, States )]
pub enum MyState
{   #[default] LoadAssets,
    InitApp,
    GameStart,
    MainLoop,
}

//Stateの遷移に使うマーカー(not Resource)
#[derive( Default )] pub struct MainLoop;

//Stateの遷移に使うResouce
#[derive( Resource )] pub struct AfterLoadAssetsTo <T: States> ( pub T );
#[derive( Resource )] pub struct AfterInitAppTo    <T: States> ( pub T );

//Stateの遷移に使うTrait
pub trait GotoState { fn next( &self ) -> MyState; }

//Traitの実装
impl GotoState for MainLoop                   { fn next( &self ) -> MyState { MyState::MainLoop } }
impl GotoState for AfterLoadAssetsTo<MyState> { fn next( &self ) -> MyState { self.0 } }
impl GotoState for AfterInitAppTo<MyState>    { fn next( &self ) -> MyState { self.0 } }

////////////////////////////////////////////////////////////////////////////////

//画面デザイン(枠)
pub struct ScreenFrame<'a>
{   pub design  : Vec<&'a str>,
    pub viewport: ViewPortInfo,
    pub minimap : MiniMapInfo,
}

//3Dカメラの表示領域(viewport)の情報
pub struct ViewPortInfo
{   pub origin: Vec2,
    pub size  : Vec2,
}

//ミニマップの情報
pub struct MiniMapInfo
{   pub zero: IVec2,
    pub size: IVec2,
}

////////////////////////////////////////////////////////////////////////////////

//四方を表す列挙型
#[derive( Default, Clone, Copy, PartialEq, Eq, Hash, Debug )]
pub enum News { #[default] North, East, West, South }

//IVec2 = IVec2 + News
impl Add<News> for IVec2
{   type Output = IVec2;
    fn add( mut self, news: News ) -> IVec2
    {   match news
        {   News::North => { self.y -= 1; }
            News::East  => { self.x += 1; }
            News::West  => { self.x -= 1; }
            News::South => { self.y += 1; }
        }
        self
    }
}

//IVec2 += News
impl AddAssign<News> for IVec2
{   fn add_assign( &mut self, news: News )
    {   match news
        {   News::North => { self.y -= 1; }
            News::East  => { self.x += 1; }
            News::West  => { self.x -= 1; }
            News::South => { self.y += 1; }
        }
    }
}

impl News
{   //四方に対応するXZ平面上の角度（四元数）を返す（Y軸回転）
    #[allow(clippy::wrong_self_convention)]
    pub fn to_quat_y( &self ) -> Quat
    {   match self
        {   News::North => Quat::from_rotation_y( PI * 0.0 ),
            News::East  => Quat::from_rotation_y( PI * 1.5 ),
            News::West  => Quat::from_rotation_y( PI * 0.5 ),
            News::South => Quat::from_rotation_y( PI * 1.0 ),
        }
    }

    //四方に対応するXY平面上の角度（四元数）を返す（Z軸回転）
    #[allow(clippy::wrong_self_convention)]
    pub fn to_quat_z( &self ) -> Quat
    {   match self
        {   News::North => Quat::from_rotation_z( PI * 0.0 ),
            News::East  => Quat::from_rotation_z( PI * 1.5 ),
            News::West  => Quat::from_rotation_z( PI * 0.5 ),
            News::South => Quat::from_rotation_z( PI * 1.0 ),
        }
    }

    //時計回りで方角を得る
    pub fn turn_right( &self ) -> Self
    {   match self
        {   News::North => News::East,
            News::East  => News::South,
            News::West  => News::North,
            News::South => News::West,
        }
    }

    //反時計回りで方角を得る
    pub fn turn_left( &self ) -> Self
    {   match self
        {   News::North => News::West,
            News::East  => News::North,
            News::West  => News::South,
            News::South => News::East,
        }
    }

    //背面の方角を得る
    pub fn back( &self ) -> Self
    {   match self
        {   News::North => News::South,
            News::East  => News::West,
            News::West  => News::East,
            News::South => News::North,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

//End of code.