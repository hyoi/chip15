use super::*;

////////////////////////////////////////////////////////////////////////////////

//Mapのメソッド（迷路作成）
impl Map
{   //空地拡張型の迷路作成メソッド
    pub fn build_maze_b( &mut self ) -> &mut Self
    {   //スタート地点を決める
        if self.start == IVec2::NEG_ONE
        {   let x = self.rng.gen_range( MAP_GRIDS_X_RANGE_INNER );
            let y = self.rng.gen_range( MAP_GRIDS_Y_RANGE_INNER );
            self.start = IVec2::new( x, y );
            let start = self.start;
            self.set_space( start );
        }

        //穴掘りループ
        let mut digable_walls = Vec::new();
        loop
        {   //マップを全面走査して拡張条件を満たす壁を記録する
            digable_walls.clear();
            for x in MAP_GRIDS_X_RANGE_INNER
            {   for y in MAP_GRIDS_Y_RANGE_INNER
                {   let cell = IVec2::new( x, y );
                    if self.is_expandable( cell )
                    {   digable_walls.push( cell )
                    }
                }
            }

            //条件を満たす壁が見つからなければ迷路完成
            if digable_walls.is_empty() { break }

            //候補の中からランダムに壊す壁を決め、空地を広げる
            let cell = digable_walls[ self.rng.gen_range( 0..digable_walls.len() ) ];
            self.set_space( cell );
        }

        self //メソッドチェーン用
    }

    //拡張条件を満たす壁か？
    fn is_expandable( &self, cell: IVec2 ) -> bool
    {   //そもそも壁ではないので掘れない
        if ! self.is_wall( cell ) { return false }

        //下向き凸の削り許可
        if   self.is_wall( cell + News::North + News::West )
        &&   self.is_wall( cell + News::North )
        &&   self.is_wall( cell + News::North + News::East )
        && ! self.is_wall( cell + News::West )
        && ! self.is_wall( cell + News::East )
        && ! self.is_wall( cell + News::South + News::West )
        && ! self.is_wall( cell + News::South )
        && ! self.is_wall( cell + News::South + News::East ) { return true }

        //右向き凸の削り許可
        if   self.is_wall( cell + News::North + News::West )
        && ! self.is_wall( cell + News::North )
        && ! self.is_wall( cell + News::North + News::East )
        &&   self.is_wall( cell + News::West )
        && ! self.is_wall( cell + News::East )
        &&   self.is_wall( cell + News::South + News::West )
        && ! self.is_wall( cell + News::South )
        && ! self.is_wall( cell + News::South + News::East ) { return true }

        //左向き凸の削り許可
        if ! self.is_wall( cell + News::North + News::West )
        && ! self.is_wall( cell + News::North )
        &&   self.is_wall( cell + News::North + News::East )
        && ! self.is_wall( cell + News::West )
        &&   self.is_wall( cell + News::East )
        && ! self.is_wall( cell + News::South + News::West )
        && ! self.is_wall( cell + News::South )
        &&   self.is_wall( cell + News::South + News::East ) { return true }

        //上向き凸の削り許可
        if ! self.is_wall( cell + News::North + News::West )
        && ! self.is_wall( cell + News::North )
        && ! self.is_wall( cell + News::North + News::East )
        && ! self.is_wall( cell + News::West )
        && ! self.is_wall( cell + News::East )
        &&   self.is_wall( cell + News::South + News::West )
        &&   self.is_wall( cell + News::South )
        &&   self.is_wall( cell + News::South + News::East ) { return true }

        //縦の貫通路になる場合はfalse
        if ! self.is_wall( cell + News::North )
        && ! self.is_wall( cell + News::South ) { return false }

        //横の貫通路になる場合はfalse
        if ! self.is_wall( cell + News::West )
        && ! self.is_wall( cell + News::East ) { return false }

        //左上が壁でなく、上と左が壁ならfalse
        if ! self.is_wall( cell + News::North + News::West )
        &&   self.is_wall( cell + News::North )
        &&   self.is_wall( cell + News::West  ) { return false }

        //右上が壁でなく、上と右が壁ならfalse
        if ! self.is_wall( cell + News::North + News::East )
        &&   self.is_wall( cell + News::North )
        &&   self.is_wall( cell + News::East  ) { return false }

        //左下が壁でなく、下と左が壁ならfalse
        if ! self.is_wall( cell + News::South + News::West )
        &&   self.is_wall( cell + News::West  )
        &&   self.is_wall( cell + News::South ) { return false }

        //右下が壁でなく、下と右が壁ならfalse
        if ! self.is_wall( cell + News::South + News::East )
        &&   self.is_wall( cell + News::East  )
        &&   self.is_wall( cell + News::South ) { return false }

        //上下左右がすべて壁はfalse（掘ると飛び地になる）
        if self.is_wall( cell + News::North )
        && self.is_wall( cell + News::West  )
        && self.is_wall( cell + News::East  )
        && self.is_wall( cell + News::South ) { return false }

        //掘削できる壁
        true
    }
}

////////////////////////////////////////////////////////////////////////////////

//End of code.