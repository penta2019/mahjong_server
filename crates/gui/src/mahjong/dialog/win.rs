use super::{super::prelude::*, *};

#[derive(Debug)]
pub struct WinDialog {
    entity: Entity,
    event: EventWin,
    camera_seat: Seat,
    next_win_index: usize,
}

impl WinDialog {
    pub fn new(event: &EventWin, camera_seat: Seat) -> Self {
        let mut obj = Self {
            entity: cmd().spawn_empty().id(),
            event: event.clone(),
            camera_seat,
            next_win_index: 0,
        };
        obj.next_win();
        obj
    }

    fn next_win(&mut self) -> bool {
        let cmd = cmd();

        cmd.entity(self.entity).despawn();
        if self.next_win_index >= self.event.contexts.len() {
            return false;
        }
        let ctx = &self.event.contexts[self.next_win_index];
        self.next_win_index += 1;

        let entity = cmd
            .spawn((
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    width: Val::Px(600.0),
                    height: Val::Px(400.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ))
            .id();

        // 手牌
        cmd.spawn((
            ChildOf(entity),
            Node {
                height: Val::Px(64.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ));

        // ドラ,裏ドラ
        cmd.spawn((
            ChildOf(entity),
            Node {
                height: Val::Px(64.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ));

        // 役
        cmd.spawn((
            ChildOf(entity),
            Node {
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .add_child(create_yaku_view(&ctx.score_context.yakus));

        // 得点
        let sctx = &ctx.score_context;
        let title_str = if sctx.title.is_empty() {
            format!("{}符{}飜", sctx.fu, sctx.fan)
        } else {
            ctx.score_context.title.clone()
        };
        let point_str = if ctx.is_drawn {
            // ツモ
            if ctx.is_dealer {
                format!("{}点オール", sctx.points.1)
            } else {
                format!("{}点・{}点", sctx.points.1, sctx.points.2)
            }
        } else {
            // ロン
            format!("{}点", sctx.points.0)
        };
        cmd.spawn((
            ChildOf(entity),
            Node {
                // height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![create_text(format!("{} {}", title_str, point_str), 24.0)],
        ));

        // OKボタン
        cmd.spawn((
            ChildOf(entity),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![create_ok_button()],
        ));

        self.entity = entity;

        true
    }
}

impl Dialog for WinDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        handle_dialog_ok_button(ok_buttons) && !self.next_win()
    }
}

fn create_yaku_view(yaku: &[Yaku]) -> Entity {
    let cmd = cmd();

    let entity = cmd
        .spawn((Node {
            // justify_content: JustifyContent::Center,
            column_gap: Val::Px(16.0),
            ..default()
        },))
        .id();

    // 役の種類は1列に5種まで. 1~5種(1列), 6~10種(2列), 10~15種(3列)
    let yakus_in_col = 5;
    let mut cols = vec![];
    for _ in 0..((yaku.len() - 1) / yakus_in_col) + 1 {
        // 役名の列
        let col_name = cmd
            .spawn((
                ChildOf(entity),
                Node {
                    width: Val::Px(128.0),
                    // margin: UiRect::left(Val::Px(32.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .id();

        // 飜数の列
        let col_fan = cmd
            .spawn((
                ChildOf(entity),
                Node {
                    // margin: UiRect::left(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd, // '飜'の文字が縦に綺麗に揃うように右寄せ
                    ..default()
                },
            ))
            .id();

        cols.push((col_name, col_fan));
    }

    for (i, y) in yaku.iter().enumerate() {
        cmd.spawn(create_text(y.name.clone(), 20.0))
            .insert(ChildOf(cols[i / yakus_in_col].0));
        cmd.spawn(create_text(y.fan.to_string() + "飜", 20.0))
            .insert(ChildOf(cols[i / yakus_in_col].1));
    }
    entity
}
