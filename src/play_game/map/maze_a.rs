use super::*;

////////////////////////////////////////////////////////////////////////////////

//Mapのメソッド（迷路作成）
impl Map
{   //穴掘り型の迷路作成メソッド１（縛りなし）
    pub fn build_maze_a_1( &mut self ) -> &mut Self
    {   //スタート地点を決める
        if self.start == IVec2::NEG_ONE
        {   let x = self.rng.gen_range( MAP_GRIDS_X_RANGE_INNER );
            let y = self.rng.gen_range( MAP_GRIDS_Y_RANGE_INNER );
            self.start = IVec2::new( x, y );
            let start = self.start;
            self.set_space( start );
        }

        //穴を掘る準備
        let mut cell = self.start;
        let mut digable_walls = Vec::new();
        let mut backtrack;

        //穴掘りループ
        loop
        {   //四方の判定準備
            digable_walls.clear();
            backtrack = IVec2::NEG_ONE;

            //四方の掘れる壁と戻り道を記録する
            for news in NEWS
            {   let next = cell + news;

                //外壁は掘れない
                if ! MAP_GRIDS_X_RANGE_INNER.contains( &next.x )
                || ! MAP_GRIDS_Y_RANGE_INNER.contains( &next.y ) { continue }

                //四方のグリッドを調べる
                if self.is_wall( next ) && self.is_digable( next, news )
                {   //壁であり且つ掘れるなら
                    digable_walls.push( next );
                }
                else if self.is_space( next ) && ! self.is_footprints( next )
                {   //道であり且つ足跡マーキングがないなら
                    backtrack = next;
                }
            }

            if ! digable_walls.is_empty()
            {   //掘れる壁が見つかったので、方向をランダムに決めて進む
                cell = digable_walls[ self.rng.gen_range( 0..digable_walls.len() ) ];
                self.set_space( cell );
            }
            else
            {   //掘れる壁が見つからず、戻り道も見つからないなら迷路完成
                if backtrack == IVec2::NEG_ONE { break }

                //現在位置に足跡をマークし、戻り路へ進む(後戻りする)
                self.add_flag_footprints( cell );
                cell = backtrack;
            }
        }

        self //メソッドチェーン用
    }

    //壁が掘れるか調べる
    fn is_digable( &self, cell: IVec2, news: News ) -> bool
    {    match news
        {   News::North
            if self.is_wall( cell + News::North + News::West )
            && self.is_wall( cell + News::North              ) // 壁壁壁
            && self.is_wall( cell + News::North + News::East ) // 壁？壁
            && self.is_wall( cell + News::West               )
                => true,
            News::West
            if self.is_wall( cell + News::North + News::West )
            && self.is_wall( cell + News::North              ) // 壁壁
            && self.is_wall( cell + News::West               ) // 壁？◎
            && self.is_wall( cell + News::South + News::West ) // 壁壁
            && self.is_wall( cell + News::South              )
                => true,
            News::East
            if self.is_wall( cell + News::North              )
            && self.is_wall( cell + News::North + News::East ) // 　壁壁
            && self.is_wall( cell + News::East               ) // ◎？壁
            && self.is_wall( cell + News::South              ) // 　壁壁
            && self.is_wall( cell + News::South + News::East )
                => true,
            News::South
            if self.is_wall( cell + News::West               )
            && self.is_wall( cell + News::East               ) // 　◎
            && self.is_wall( cell + News::South + News::West ) // 壁？壁
            && self.is_wall( cell + News::South              ) // 壁壁壁
            && self.is_wall( cell + News::South + News::East )
                => true,
            _   => false,
        }
    }

    //穴掘り型の迷路作成メソッド２（四方の外壁全てにタッチすると終了）
    pub fn build_maze_a_2( &mut self ) -> &mut Self
    {   //スタート地点を決める
        if self.start == IVec2::NEG_ONE
        {   let x = self.rng.gen_range( MAP_GRIDS_X_RANGE_INNER );
            let y = self.rng.gen_range( MAP_GRIDS_Y_RANGE_INNER );
            self.start = IVec2::new( x, y );
            let start = self.start;
            self.set_space( start );
        }

        //穴を掘る準備
        let mut cell = self.start;
        let mut digable_walls = Vec::new();
        let mut backtrack;
        let mut flags = HashSet::new();

        //穴掘りループ
        loop
        {   //四方の判定準備
            digable_walls.clear();
            backtrack = IVec2::NEG_ONE;

            //四方の掘れる壁と戻り道を記録する
            for news in NEWS
            {   let next = cell + news;

                //外壁は掘れない
                if ! MAP_GRIDS_X_RANGE_INNER.contains( &next.x )
                || ! MAP_GRIDS_Y_RANGE_INNER.contains( &next.y ) { continue }

                //四方のグリッドを調べる
                if self.is_wall( next ) && self.is_digable( next, news )
                {   //壁であり且つ掘れるなら
                    digable_walls.push( next );
                }
                else if self.is_space( next ) && ! self.is_footprints( next )
                {   //道であり且つ足跡マーキングがないなら
                    backtrack = next;
                }
            }

            if ! digable_walls.is_empty()
            {   //掘れる壁が見つかったので、方向をランダムに決めて進む
                cell = digable_walls[ self.rng.gen_range( 0..digable_walls.len() ) ];
                self.set_space( cell );

                //外壁に達したら
                if cell.y <= 1                    { flags.insert( News::North ); }
                if cell.x >= MAP_GRIDS_WIDTH  - 2 { flags.insert( News::East  ); }
                if cell.x <= 1                    { flags.insert( News::West  ); }
                if cell.y >= MAP_GRIDS_HEIGHT - 2 { flags.insert( News::South ); }

                if flags.len() >= 4 { break } //四方の外壁すべてに達したらループ脱出
            }
            else
            {   //掘れる壁が見つからず、戻り道も見つからないなら迷路完成
                if backtrack == IVec2::NEG_ONE { break }

                //現在位置に足跡をマークし、戻り路へ進む(後戻りする)
                self.add_flag_footprints( cell );
                cell = backtrack;
            }
        }

        self //メソッドチェーン用
    }
}

////////////////////////////////////////////////////////////////////////////////

//End of code.