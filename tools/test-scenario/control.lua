local texts = {
  "纯中文：这是用于测试自动换行的中文描述，应该按照实际字体宽度换行，标点不能出现在错误的位置。",
  "混排：使用Factorio自动化生产IronPlate和CopperCable，每分钟1200个物品，支持UTF-8中文English混合排版。",
  "图标：前文前文前文[item=iron-plate]后文后文后文[entity=fusion-reactor]后面的文字不应粘连。",
  "链接：前文前文前文[item=iron-plate,quality=legendary]后文后文后文[fluid=water]继续测试。",
  "English words should wrap at word boundaries. WWW iii 12345 hello world.",
  "标点测试：“中文（测试）”，不应该让，。！？：；）】出现在行首，也不能让（【出现在行尾。",
  "第一行\n\n第三行 [color=red]红色中文English混排[/color] 最后结尾。"
}
local function panel(p)
 if p.gui.screen.wraptest then p.gui.screen.wraptest.destroy() end
 local f=p.gui.screen.add{type="frame",name="wraptest",caption="Word wrap runtime regression",direction="vertical"}
 f.auto_center=true
 local buttons=f.add{type="flow"}
 buttons.add{type="button",name="encyclopedia",caption="聚变反应堆百科"}
 buttons.add{type="button",name="combinator",caption="运算器描述"}
 buttons.add{type="button",name="refresh",caption="重新排版"}
 local grid=f.add{type="table",column_count=3}
 for _,w in ipairs{160,240,360} do
  local col=grid.add{type="flow",direction="vertical"}
  col.add{type="label",caption="Width "..w}
  for _,t in ipairs(texts) do
   local l=col.add{type="label",caption=t}
   l.style.width=w; l.style.single_line=false
  end
 end
end
script.on_event(defines.events.on_player_created,function(e)
 local p=game.get_player(e.player_index)
 p.teleport({0,0}); p.cheat_mode=true
 p.surface.request_to_generate_chunks({0,0},1); p.surface.force_generate_chunk_requests()
 storage.combinator=p.surface.create_entity{name="arithmetic-combinator",position={2,0},force=p.force}
 storage.combinator.combinator_description="中文Factorio混排：前面的文字前面的文字[entity=fusion-generator]后面的中文EnglishWord不能粘连。使用[item=iron-plate]和[fluid=water]继续生产，标点不能，。！？出现在行首。"
 p.gui.top.add{type="button",name="showtest",caption="换行测试"}
 panel(p)
end)
script.on_event(defines.events.on_gui_click,function(e)
 local p=game.get_player(e.player_index)
 if e.element.name=="showtest" or e.element.name=="refresh" then panel(p)
 elseif e.element.name=="encyclopedia" then
  p.gui.screen.wraptest.destroy(); p.open_factoriopedia_gui(prototypes.entity["fusion-reactor"])
 elseif e.element.name=="combinator" then
  p.gui.screen.wraptest.destroy(); p.opened=storage.combinator
 end
end)

-- Loopback-only test control. Enable explicitly with --enable-lua-udp 34210.
script.on_event(defines.events.on_tick,function() helpers.recv_udp() end)
script.on_event(defines.events.on_udp_packet_received,function(e)
 local p=game.get_player(1)
 if not p then return end
 local cmd=helpers.json_to_table(e.payload)
 if not cmd then return end
 p.close_factoriopedia_gui(); p.opened=nil
 if p.gui.screen.wraptest then p.gui.screen.wraptest.destroy() end
 if p.gui.screen.stress then p.gui.screen.stress.destroy() end
 if cmd.command=="panel" then panel(p)
 elseif cmd.command=="encyclopedia" then p.open_factoriopedia_gui(prototypes.entity["fusion-reactor"])
 elseif cmd.command=="combinator" then p.opened=storage.combinator
 elseif cmd.command=="stress" then
  local frame=p.gui.screen.add{type="frame",name="stress",caption="Native font regression matrix",direction="vertical"}
  frame.auto_center=true
  local scroll=frame.add{type="scroll-pane",direction="vertical"}
  scroll.style.maximal_height=800; scroll.style.width=800
  for _,font in ipairs{"default","default-small","default-large"} do
   for _,w in ipairs{32,64,96,128,160,240,360} do
    for _,t in ipairs(texts) do
     local l=scroll.add{type="label",caption=t}
     l.style.font=font; l.style.width=w; l.style.single_line=false
    end
   end
  end
  local l=scroll.add{type="label",caption=string.rep("长文本中文Factorio[item=iron-plate]，",128)}
  l.style.width=360; l.style.single_line=false
  -- Explicit height clipping must retain Factorio's TooLong result semantics.
  local clipped=scroll.add{type="label",caption=string.rep("高度限制中文English ",32)}
  clipped.style.width=160; clipped.style.maximal_height=32; clipped.style.single_line=false
  for _,setting in ipairs{defines.rich_text_setting.enabled,defines.rich_text_setting.disabled} do
   local box=scroll.add{type="text-box",text=storage.combinator.combinator_description}
   box.style.width=240; box.style.height=112; box.style.rich_text_setting=setting
  end
 end
 log("wordwrap-test command="..cmd.command.." tick="..game.tick)
 helpers.send_udp(e.source_port,helpers.table_to_json{ok=true,command=cmd.command,tick=game.tick})
end)

