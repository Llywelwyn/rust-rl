## a roguelike in rust, playable @ [llywelwyn.github.io](https://llywelwyn.github.io/)

#### using _rltk/bracket-lib_, and _specs_

![image](https://github.com/Llywelwyn/rust-rl/assets/82828093/b05e4f0b-2062-4abe-9fee-c679f9ef420d)

this year for roguelikedev does the complete tutorial, i'm following along with thebracket's [_roguelike tutorial - in rust_](https://bfnightly.bracketproductions.com). for most of the 8 weeks, i'll probably just be working through the content rather than diverging too much into doing my own thing, since it's lengthy and i'd rather finish in time. that said, the ultimate aim here is to strip out the vast majority of the existing entities and replace them with my own, using the systems and components from the tutorial as a jumping-off point for something of my own making.

i'll try to remember to update the web version on my page at the end of every week

---

<details>
<summary>week 1</summary>
  
- brogue-like colours
  - i was staring at a horrible-looking game for a while as i tried to figure out how to make it look nice, before deciding to try the brogue method of colour offsets. when a map is generated, it also generates a red, green, and blue offset value for every tile on the map, and applies them during rendering. after making that change i started to miss the previous hue, so i combined the two. as it stands, every tile starts off a subtle green/blue, has rgb offsets applied on top of that, and then has the actual tile colour applied. and it ends up making something like this

    ![image](https://github.com/Llywelwyn/rust-rl/assets/82828093/2ded4eb7-b758-4022-8fee-fdf12673cf0e)

- fov
  - decided to use bracket-lib's symmetric shadowcasting for common viewsheds (i.e. sight)
  - and implemented elig's [raycasting](https://www.roguebasin.com/index.php/Eligloscode) algorithm for any viewsheds that _dont_ need that level of detail. symmetric is great, but when it comes to viewsheds that often _aren't_ symmetric in the first place, it's not really necessary (i.e. it's not often you've got two people with: the same additional viewshed, both within range, etc.). doing it this way comes with the benefit of being able to easily define what blocks a viewshed, rather than having to make a whole new BaseMap to work through bracket-lib

- telepaths and having brains
  - telepathy! a personal favourite rl feature, so i thought it'd be a cool test of the raycasting. right now it's simple, since the point was really just making sure the raycasting worked: there's a component for _being a telepath_, and for _having a mind_. if someone has telepathy, they'll see every entity with a mind within a given radius (defined by their telepath component), even through walls.

    ![image](https://github.com/Llywelwyn/rust-rl/assets/82828093/d55d5df4-267c-4dd5-b166-8417f58365af)
    
- atomised spawn tables
  - i tried figuring out how often things would spawn by just looking at the weighted tables, and i had no idea at a glance, so i replaced it with category tables. right now it's just rolling for an entity or a mob, and then rolling on the right table from there, but at least it means easily being able to see how often something will spawn. on average right now, there's 1 item : 3 mobs

</details>

---

<details>
  <summary>week 2</summary>
  
- most of section 3 - generating maps
  - this week was mostly just working away at the mapgen stuff. getting all the algorithms in, chaining builders, being able to do prefabs. whenever i got bored i just opened rexpaint and toyed around with making simple vaults.
  
- 8-bit walls
  - i wasn't happy with how the walls looked, so i made the masks 8-bit instead of just 4-, which means being able to be a lot more specific with which glyphs are used. mainly it means no more grids of ╬. this comes with a side-effect of magic mapping looking a lot better.

    ![wall bitmask before-and-after](https://github.com/Llywelwyn/rust-rl/assets/82828093/6568d203-e0b0-4c68-ad81-fe2d5c2f0ac3)

</details>

---

<details>
  <summary>week 3</summary>

- (better) vault loot
  - moved over to using raws and atomised spawn tables into a bunch of sub-categories in the process, like wands, equipment, potions, etc. now there's options for rolling just out of subsets of items - useful for adding a specific spawn to a vault, or ensuring there's always an amount of food on a given level, etc. can also use this in the future for categorising groups of mobs, to only spawn x mobtype on a given map too.
  
    ![image](https://github.com/Llywelwyn/rust-rl/assets/82828093/32b73494-2d70-424f-a551-fe911c66ef9b)


- actions with directions
  - made a new runstate that prompts the player to pick a direction, and takes a function as an argument. after the player picks a direction, it calls the function with that direction as the args. right now it's being used for door stuff, but now it'll be super easy to make anything else that needs the same parameters
    
    ![week 3 - kicking doors](https://github.com/Llywelwyn/rust-rl/assets/82828093/561135cc-87ae-4e19-b065-486c3736542d)


- ui stuff
  - there's a help screen now with controls, accessed with [?], and a death screen that actually logs some stuff
    
    ![image](https://github.com/Llywelwyn/rust-rl/assets/82828093/cedd471d-8f5c-4a94-9ea1-6999fc56372d)
  - finally, identical items in the inventory stack. i waited with this until figuring out a way that would work with extra parameters in the future like BUC. current solution is using a BTreeMap with a tuple containing the parameters that need to be the same (right now just the name) as the key, and the number of those items in the inventory as the value.
  - wand uses are tracked now with a number of asterisks next to their name -- i'll change this once i add in identification
    
    ![image](https://github.com/Llywelwyn/rust-rl/assets/82828093/98d15bee-e825-47ea-9ef8-04d8312f00af)

</details>

---

<details>
  <summary>week 4</summary>

- d20/hacklike combat overhaul
  - the framework for levels, attributes, and skills are all in, as well as a lot of the associated systems. it now uses a system that trends way closer to the -hack brand of roguelikes (it's almost identical). i thought about doing something more novel just because then i could say i made it on my own, but then i decided i'd rather lean on the 35 years of balance tweaks nethack has had than start all over from scratch. not having to worry so much about balance gives me time to do other stuff, and i think the familiarity for players will be nice too. my favourite addition is a MULTIATTACK flag for mobs - if they have it, they'll use all their natural attacks in a turn instead of picking a random one
 
- extremely free-form loot tables (like rats spawning... lambs?)
  - i realised my loot table structure wasn't very different from the spawn tables i'd been using for mapgen. other than one field, the structures were identical, so i decided to massively generalise how loot spawning works. instead of only allowing mobs to drop items from the specifically defined loot tables, they now have the capability to drop _anything_ from _any_ table -- for example, an animal can drop stuff from the animal drop table, or it could be set to drop a random scroll, or literally any other entity... including other mobs! i decided to test this with rats that had a 25% chance to "drop" anything from the _mobs_ spawn table on death. in this case, one rat left behind a lamb, and another left behind a fawn.
 
    ![image](https://github.com/Llywelwyn/rust-rl/assets/82828093/b4c79e09-e8a7-4303-a9e8-bee03afb7afe)

- and a huge visual overhaul!
  - a whole new ui, a new font (a 14x16 curses variant), a system to spawn particles on a delay for proper - if basic - animation, and a couple new features to fill in the expanded ui space (like being able to see a list of entities in view on the sidebar).

   ![week 4 - visual overhaul](https://github.com/Llywelwyn/rust-rl/assets/82828093/8b6485af-a7a5-4102-9df1-896538cf8e50)

</details>

---

<details>
  <summary>week 5</summary>

- not a whole lot
  - busy week for me, so the only things i managed this week were finishing up some small things from the previous week. i started on combining logs that happen on the same tick (You hit the goblin. The goblin hits you.), and fixed up the _chance to spawn a guy each turn_ system to work with the groupsize flags, so there's a chance to spawn a pack of creatures each game turn rather than packs only spawning on map generation.

</details>

---

<details>
  <summary>week 6</summary>

- visuals (contd.)
  - i'll start here, because it pretty much shows all the other things i did this week in the one image. the biggest change is nixing the rainbow colours i did in week 4 - having all the names on the sidebar and inventory be coloured by the colour of their glyph made for a difficult to parse ui. i swapped everything out for being coloured by their rarity (white for common and nonmagical items, grey for unidentified, and green-to-gold for uncommon-legendary rarities), and decided to add the glyph of the entity beside the name. overall, a big win i think. it provides more information more cleanly.
  
  ![image](https://github.com/Llywelwyn/rust-rl/assets/82828093/af1c7821-d32d-4914-82f1-4422e1bef680)

- backtracking
  - short and sweet, maps are persistent now. entities don't get deleted when you descend, they just get frozen, and you can return anytime. stair-dancing is a thing now, and something i haven't decided how to handle yet.
 
- item identification (no BUC yet!)
  - unique schemes for unidentified items in the big categories (i.e. "effervescent green potions", "octagonal wands", and "GIB BERISH scrolls"), as well as the option of defining one-off obfuscated names for special cases

- encumbrance
  - varying levels of being overweight, with the limit determined by strength, slowing entities down by ~25% per level over unencumbered. right now it's pretty forgiving, and i'd probably like it to stay that way. my ideal balance here would be roughly how it ends up in 5e DnD: everyone can carry a good supply of consumables, but strength characters usually don't carry a much higher quantity of items than anyone else, because the strongest armour is extremely heavy. sort of like a soft strength requirement for the heaviest gear - rather than requiring specific stats or levels to equip it, it's heavy enough that you need to be strong to lug it around. but if someone weaker wanted to, they could, they'd just have to leave some other items behind to do so. or take the speed penalty for being encumbered

</details>

---

<details>
  <summary>week 7</summary>

- character creation!
  - it doesn't look too pretty yet, but i managed to get it done in time. classes and ancestries are selectable, with ancestries determining some intrinsic bonuses (dwarves are hardy, catfolk are fast and have claws, etc.) and attribute maximums, and classes determining starting equipment and the actual attribute rolls. along with this, i expanded entity reactions - now a shared ancestry is taken into account first of all, and it checks faction if it doesn't manage to find anything. this means humans wont attack other humans, dwarves wont be attacked by gnomes and other dwarves, etc.
  
  [image here]

- better AI
   - straight from thebracket, with a handful of tweaks of my own, i've finally atomised AI into adjacent, visible, chase, flee, and default systems. most notably,rather than hostile mobs attacking everything and passive mobs doing nothing, every mob has a faction, and most mobs have an ancestry. like mentioned above, mobs will take all this into account when determining how they want to react to any other entity. i see a lot of places to expand on this in the future, like going into alignments and other increasingly-specific reasons why any entity might want to murder another. or make friends with them. taming comes to mind here.
 
- an effects system
  - instead of randomly doing things all around the codebase, everything is in the process of being moved over to an effects system. to put it very simply, there's a big list of every effect that needs to take place on the next tick, and each tick the queue is iterated through, the next effect is checked against a list of every entity that died this turn to make sure that it should still take place (no dead mobs still getting their attacks off), and then it makes the event happen if appropriate. if not, it just gets tossed out. it's all super modular, so effects can pretty much be applied to everything. the same damage and targeting effects work for item use and traps going off, or an entity swinging their sword, for example. i made use of this new system by adding in some aoe scrolls, like mass function and mass healing.

- better logging
  - it's still in process, but i'm making every entity name in chat use the proper colour now, and working with the effects system to combine relevant logs all into one line, and make them more specific. generally i'm trying not to include too many numbers in the log; it's just personal preference, but i don't like seeing exactly how much damage every hit has done - i'd rather just see my health bar go up or down, or tell by seeing how fast a mob dies. this took *too much* fumbling with the function that renders my log to the screen, but now it's fixed and works a lot better than before, so it was definitely worth it.

  [image here]

</details>
