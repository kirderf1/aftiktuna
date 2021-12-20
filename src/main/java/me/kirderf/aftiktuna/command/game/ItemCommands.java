package me.kirderf.aftiktuna.command.game;

import com.mojang.brigadier.arguments.StringArgumentType;
import me.kirderf.aftiktuna.action.EnterDoorAction;
import me.kirderf.aftiktuna.action.ForceDoorAction;
import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.CommandUtil;
import me.kirderf.aftiktuna.object.Item;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.ai.TakeItemsTask;
import me.kirderf.aftiktuna.object.entity.ai.WieldTask;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.object.type.WeaponType;

import java.util.Optional;

import static me.kirderf.aftiktuna.command.game.GameCommands.*;

public final class ItemCommands {
	static void register() {
		DISPATCHER.register(literal("take")
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> takeItem(context.getSource(), ObjectArgument.getType(context, "item"))))
				.then(literal("items").executes(context -> takeItems(context.getSource()))));
		DISPATCHER.register(literal("wield")
				.executes(context -> wieldBestWeapon(context.getSource()))
				.then(argument("item", ObjectArgument.create(ObjectTypes.WEAPONS))
						.executes(context -> wieldItem(context.getSource(), ObjectArgument.getType(context, "item", WeaponType.class)))));
		DISPATCHER.register(literal("give").then(argument("name", StringArgumentType.string())
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS)).executes(context -> giveItem(context.getSource(),
						StringArgumentType.getString(context, "name"), ObjectArgument.getType(context, "item", ItemType.class))))));
		DISPATCHER.register(literal("use")
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> useItem(context.getSource(), ObjectArgument.getType(context, "item", ItemType.class)))));
		DISPATCHER.register(literal("examine")
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
						.executes(context -> examineItem(context.getSource(), ObjectArgument.getType(context, "item", ItemType.class)))));
	}
	
	private static int takeItem(CommandContext context, ObjectType type) {
		Aftik aftik = context.getControlledAftik();
		return CommandUtil.searchForAccessible(context, aftik, Item.CAST.filter(type::matching), true,
				item -> context.action(out -> aftik.moveAndTake(item, out)),
				() -> context.printNoAction("There is no %s here to pick up.", type.name()));
	}
	
	private static int takeItems(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (aftik.isAnyNearAccessible(Item.CAST.toPredicate(), true)) {
			return context.action(out -> aftik.getMind().setAndPerformPlayerTask(new TakeItemsTask(), out));
		} else {
			return context.printNoAction("There are no nearby items to take.");
		}
	}
	
	private static int wieldItem(CommandContext context, WeaponType weaponType) {
		Aftik aftik = context.getControlledAftik();
		
		if (aftik.hasItem(weaponType)) {
			return context.action(out -> aftik.wieldFromInventory(weaponType, out));
		} else {
			return CommandUtil.searchForAccessible(context, aftik, Item.CAST.filter(weaponType::matching), true,
					item -> context.action(out -> aftik.moveAndWield(item, weaponType, out)),
					() -> context.printNoAction("There is no %s that %s can wield.", weaponType.name(), aftik.getName()));
		}
	}
	
	private static int wieldBestWeapon(CommandContext context) {
		Aftik aftik = context.getControlledAftik();
		
		Optional<WeaponType> weaponType = WieldTask.findWieldableInventoryItem(aftik);
		
		return weaponType.map(type -> context.action(out -> aftik.wieldFromInventory(type, out)))
				.orElseGet(() -> context.printNoAction("%s does not have a better item to wield.", aftik.getName()));
	}
	
	private static int giveItem(CommandContext context, String name, ItemType type) {
		Aftik aftik = context.getControlledAftik();
		Optional<Aftik> aftikOptional = context.getCrew().findByName(name);
		
		if (aftikOptional.isPresent() && aftik.getArea() == aftikOptional.get().getArea()) {
			Aftik target = aftikOptional.get();
			
			if (aftik == target) {
				return context.printNoAction("%s can't give an item to themselves.", aftik.getName());
			}
			
			if (aftik.hasItem(type)) {
				return CommandUtil.ifAccessible(context, aftik, target.getPosition(),
						() -> context.action(out -> aftik.moveAndGive(target, type, out)));
			} else {
				return context.printNoAction("%s does not have that item.", aftik.getName());
			}
		} else {
			return context.printNoAction("There is no such aftik in the area.");
		}
	}
	
	private static int useItem(CommandContext context, ItemType type) {
		Aftik aftik = context.getControlledAftik();
		if (aftik.hasItem(type)) {
			
			if (type == ObjectTypes.FUEL_CAN) {
				return launchShip(context);
			} else if (type == ObjectTypes.MEDKIT) {
				return useMedKit(context, aftik);
			} else if (type == ObjectTypes.KEYCARD) {
				return useKeycard(context, aftik);
			} else if (type.getForceMethod() != null) {
				return useTool(context, aftik, type);
			} else if (type instanceof WeaponType weapon) {
				return context.action(out -> aftik.wieldFromInventory(weapon, out));
			} else {
				return context.printNoAction("The item cannot be used in a meaningful way.");
			}
		} else {
			return context.printNoAction("%s does not have that item.", aftik.getName());
		}
	}
	
	private static int useMedKit(CommandContext context, Aftik aftik) {
		if (aftik.getHealth() < aftik.getMaxHealth()) {
			return context.action(out -> {
				if (aftik.removeItem(ObjectTypes.MEDKIT)) {
					aftik.restoreHealth(0.5F);
					out.print("%s used a medkit and recovered some health.", aftik.getName());
				}
			});
		} else {
			return context.printNoAction("%s is not hurt, and does not need to use the medkit.", aftik.getName());
		}
	}
	
	private static int useKeycard(CommandContext context, Aftik aftik) {
		Optional<Door> doorOptional = aftik.findNearestAccessible(Door.CAST.filter(door -> door.getProperty() == DoorProperty.LOCKED), true);
		if (doorOptional.isPresent()) {
			Door door = doorOptional.get();
			return context.action(out -> EnterDoorAction.moveAndEnter(aftik, door, out));
		} else {
			return context.printNoAction("There is no accessible door here that require a keycard.");
		}
	}
	
	private static int useTool(CommandContext context, Aftik aftik, ItemType item) {
		Optional<Door> doorOptional = ForceDoorAction.findForceTargetForTool(aftik, item);
		if (doorOptional.isPresent()) {
			Door door = doorOptional.get();
			return context.action(out -> ForceDoorAction.moveAndForce(aftik, door, item, out));
		} else {
			return context.printNoAction("There is no accessible door here which a %s might force open.", item.name());
		}
	}
	
	private static int examineItem(CommandContext context, ItemType type) {
		Aftik aftik = context.getControlledAftik();
		if (aftik.hasItem(type) || aftik.isAnyNear(type::matching)) {
			return context.printNoAction(type.getExamineText());
		} else {
			return context.printNoAction("There is no such item here.");
		}
	}
}