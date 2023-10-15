use super::*;

////////////////////////////////////////////////////////////////////////////////

//&mut ChildBuilder<'_, '_, '_>用メソッド
pub trait AddMethodToChildBuilder
{   //Playerの姿をspawnする
    fn spawn_player_figure
    (   &mut self,
        meshes   : &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>
    );

    //鍵付き宝箱をspawnする
    fn spawn_locked_chest
    (   &mut self,
        position : Vec3,
        rotation : Quat,
        meshes   : &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    );
}

pub trait AddMethodToChildBuilderWith<T> //ジェネリクス付き
{   //Player用3Dカメラをspawnする
    fn spawn_player_camera3d
    (   &mut self,
        component: T,
        is_active: bool,
        position : Vec3,
        target   : Vec3
    );
}

impl AddMethodToChildBuilder for &mut ChildBuilder<'_, '_, '_>
{   //Playerの姿をspawnする
    fn spawn_player_figure
    (   &mut self,
        meshes   : &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    )
    {   //配置
        self.spawn( PbrBundle::default() )
        .insert( meshes.add( shape::UVSphere { radius: 0.4, ..default() }.into() ) )
        .insert( materials.add( Color::DARK_GRAY.into() ) )
        .insert( Transform::from_translation( Vec3::ZERO ) )
        ;
        self.spawn( PbrBundle::default() )
        .insert( meshes.add( shape::UVSphere { radius: 0.395, ..default() }.into() ) )
        .insert( materials.add( Color::YELLOW.into() ) )
        .insert( Transform::from_translation( Vec3::NEG_Z * 0.01 ) )
        ;
    }

    //鍵付き宝箱をspawnする
    fn spawn_locked_chest
    (   &mut self,
        position : Vec3,
        rotation : Quat,
        meshes   : &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    )
    {   self.spawn( PbrBundle::default() )
        .insert( materials.add( Color::NONE.into() ) ) //透明
        .insert( Transform::from_translation( position ).with_rotation( rotation ) )
        .with_children
        (   | cmds |
            {   //本体
                let shape_box = shape::Box::new( 0.7, 0.3, 0.4 );
                cmds.spawn( PbrBundle::default() )
                .insert( meshes.add( shape_box.into() ) )
                .insert( materials.add( Color::MAROON.into() ) )
                .insert( Transform::from_translation( Vec3::Y * -0.35 ) )
                ;

                //上蓋
                let shape_cylinder = shape::Cylinder { height: 0.695, radius: 0.195, ..default() };
                let translation = Vec3::Y * -0.2;
                let rotation = Quat::from_rotation_z( PI * 0.5 );
                let transform = Transform::from_translation( translation ).with_rotation( rotation );
                cmds.spawn( PbrBundle::default() )
                .insert( meshes.add( shape_cylinder.into() ) )
                .insert( materials.add( Color::MAROON.into() ) )
                .insert ( transform );

                //錠前
                let shape_cube = shape::Cube::new( 0.1 );
                cmds.spawn( PbrBundle::default() )
                .insert( meshes.add( shape_cube.into() ) )
                .insert( Transform::from_translation( Vec3::Y * -0.2 + Vec3::Z * -0.17 ) )
                .insert( materials.add( Color::GRAY.into() ) )
                .with_children
                (   | cmds |
                    {   //鍵穴
                        let cylinder = shape::Cylinder { height: 0.11, radius: 0.01, ..default() };
                        cmds.spawn( PbrBundle::default() )
                        .insert( meshes.add( cylinder.into() ) )
                        .insert
                        (   Transform::from_translation( Vec3::Y * 0.02 )
                                .with_rotation( Quat::from_rotation_x( PI * 0.5 ) )
                        )
                        .insert( materials.add( Color::BLACK.into() ) );

                        let shape_box = shape::Box::new( 0.01, 0.04, 0.11 );
                        cmds.spawn( PbrBundle::default() )
                        .insert( meshes.add( shape_box.into() ) )
                        .insert( Transform::from_translation( Vec3::Y * 0.0 ) )
                        .insert( materials.add( Color::BLACK.into() ) );
                    }
                );
            }
        );
    }
}

impl<T: Component> AddMethodToChildBuilderWith<T> for &mut ChildBuilder<'_, '_, '_>
{   //Player用3Dカメラをspawnする
    fn spawn_player_camera3d
    (   &mut self,
        component: T,
        is_active: bool,
        position : Vec3,
        target   : Vec3
    )
    {   //表示領域の準備
        let viewport = Some
        (   camera::Viewport
            {   physical_position: SCREEN_FRAME.viewport.origin.as_uvec2(),
                physical_size    : SCREEN_FRAME.viewport.size.as_uvec2(),
                ..default()
            }
        );

        //配置
        self.spawn( ( Camera3dBundle::default(), component ) )
        .insert( Camera { order: ORDER_CAMERA3D_PLAYER, viewport, is_active, ..default() } )
        .insert( Camera3d { clear_color: CAMERA3D_BGCOLOR, ..default() } )
        .insert( Transform::from_translation( position ).looking_at( target, Vec3::Y ) );
    }
}

////////////////////////////////////////////////////////////////////////////////

//End of code.