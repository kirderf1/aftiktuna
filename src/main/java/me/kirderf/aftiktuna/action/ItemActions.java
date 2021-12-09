package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.arguments.StringArgumentType;
import me.kirderf.aftiktuna.object.*;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.ai.WieldTask;

import java.util.Optional;

import static me.kirderf.aftiktuna.action.ActionHandler.*;

public final class ItemActions {
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
	}
	
	private static int takeItem(InputActionContext context, ObjectType type) {
		Aftik aftik = context.getControlledAftik();
		return ActionHandler.searchForAccessible(context, aftik, Item.CAST.filter(type::matching), true,
				item -> context.action(out -> aftik.moveAndTake(item, out)),
				() -> context.printNoAction("There is no %s here to pick up.", type.name()));
	}
	
	private static int takeItems(InputActionContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (aftik.isAnyNearAccessible(Item.CAST.toPredicate(), true)) {
			return context.action(out -> aftik.getMind().setTakeItems(out));
		} else {
			return context.printNoAction("There are no nearby items to take.");
		}
	}
	
	private static int wieldItem(InputActionContext context, WeaponType weaponType) {
		Aftik aftik = context.getControlledAftik();
		
		if (aftik.hasItem(weaponType)) {
			return context.action(out -> aftik.wieldFromInventory(weaponType, out));
		} else {
			return ActionHandler.searchForAccessible(context, aftik, Item.CAST.filter(weaponType::matching), true,
					item -> context.action(out -> aftik.moveAndWield(item, weaponType, out)),
					() -> context.printNoAction("There is no %s that %s can wield.", weaponType.name(), aftik.getName()));
		}
	}
	
	private static int wieldBestWeapon(InputActionContext context) {
		Aftik aftik = context.getControlledAftik();
		
		Optional<WeaponType> weaponType = WieldTask.findWieldableInventoryItem(aftik);
		
		return weaponType.map(type -> context.action(out -> aftik.wieldFromInventory(type, out)))
				.orElseGet(() -> context.printNoAction("%s does not have a better item to wield.", aftik.getName()));
	}
	
	private static int giveItem(InputActionContext context, String name, ItemType type) {
		Aftik aftik = context.getControlledAftik();
		Optional<Aftik> aftikOptional = context.getCrew().findByName(name);
		
		if (aftikOptional.isPresent() && aftik.getArea() == aftikOptional.get().getArea()) {
			Aftik target = aftikOptional.get();
			
			if (aftik == target) {
				return context.printNoAction("%s can't give an item to themselves.", aftik.getName());
			}
			
			if (aftik.hasItem(type)) {
				return ActionHandler.ifAccessible(context, aftik, target.getPosition(),
						() -> context.action(out -> aftik.moveAndGive(target, type, out)));
			} else {
				return context.printNoAction("%s does not have that item.", aftik.getName());
			}
		} else {
			return context.printNoAction("There is no such aftik in the area.");
		}
	}
	
	private static int useItem(InputActionContext context, ItemType type) {
		Aftik aftik = context.getControlledAftik();
		if (aftik.hasItem(type)) {
			
			if (type == ObjectTypes.FUEL_CAN) {
				return launchShip(context);
			} else if (type instanceof WeaponType weapon) {
				return context.action(out -> aftik.wieldFromInventory(weapon, out));
			} else {
				return context.printNoAction("The item cannot be used in a meaningful way.");
			}
		} else {
			return context.printNoAction("%s does not have that item.", aftik.getName());
		}
	}
}