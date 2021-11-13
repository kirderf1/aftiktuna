package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.StringArgumentType;
import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.*;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.Entity;
import me.kirderf.aftiktuna.util.Either;

import java.util.Optional;
import java.util.function.Consumer;

import static me.kirderf.aftiktuna.action.ActionHandler.*;

public final class ItemActions {
	static void register(CommandDispatcher<GameInstance> dispatcher) {
		dispatcher.register(literal("take").then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS))
				.executes(context -> takeItem(context.getSource(), ObjectArgument.getType(context, "item")))));
		dispatcher.register(literal("wield").then(argument("item", ObjectArgument.create(ObjectTypes.WEAPONS))
				.executes(context -> wieldItem(context.getSource(), ObjectArgument.getType(context, "item", WeaponType.class)))));
		dispatcher.register(literal("give").then(argument("name", StringArgumentType.string())
				.then(argument("item", ObjectArgument.create(ObjectTypes.ITEMS)).executes(context -> giveItem(context.getSource(),
						StringArgumentType.getString(context, "name"), ObjectArgument.getType(context, "item"))))));
	}
	
	private static int takeItem(GameInstance game, ObjectType type) {
		Aftik aftik = game.getAftik();
		return searchForAndIfNotBlocked(game, aftik, type,
				item -> takeItem(game, aftik, item),
				() -> game.out().printf("There is no %s here to pick up.%n", type.name()));
	}
	
	private static void takeItem(GameInstance game, Aftik aftik, Item item) {
		Optional<Entity.MoveFailure> result = aftik.moveAndTake(item);
		
		result.ifPresentOrElse(
				failure -> ActionHandler.printMoveFailure(game, failure),
				() -> game.out().printf("%s picked up the %s.%n", aftik.getName(), item.getType().name()));
	}
	
	private static int wieldItem(GameInstance game, WeaponType weaponType) {
		Aftik aftik = game.getAftik();
		
		if (aftik.hasItem(weaponType)) {
			aftik.wieldFromInventory(weaponType, new ContextPrinter(game));
			return 1;
		} else {
			return searchForAndIfNotBlocked(game, aftik, weaponType,
					item -> wieldItem(game, aftik, item, weaponType),
					() -> game.out().printf("There is no %s that %s can wield.%n", weaponType.name(), aftik.getName()));
		}
	}
	
	private static void wieldItem(GameInstance game, Aftik aftik, Item item, WeaponType type) {
		Optional<Entity.MoveFailure> result = aftik.moveAndWield(item, type);
		
		result.ifPresentOrElse(
				failure -> ActionHandler.printMoveFailure(game,failure),
				() -> game.out().printf("%s picked up and wielded the %s.%n", aftik.getName(), type.name()));
	}
	
	private static int giveItem(GameInstance game, String name, ObjectType type) {
		Aftik aftik = game.getAftik();
		Optional<Aftik> aftikOptional = game.getCrew().findByName(name);
		
		if (aftikOptional.isPresent() && aftik.getRoom() == aftikOptional.get().getRoom()) {
			Aftik target = aftikOptional.get();
			
			if (aftik == target) {
				game.out().printf("%s can't give an item to themselves.%n", aftik.getName());
				return 0;
			}
			
			if (aftik.hasItem(type)) {
				
				Either<Boolean, Entity.MoveFailure> result = aftik.moveAndGive(target, type);
				
				result.run(success -> {
					if (success)
						game.out().printf("%s gave %s a %s.%n", aftik.getName(), target.getName(), type.name());
					else
						game.out().printf("%s does not have that item.%n", aftik.getName());
				}, moveFailure -> printMoveFailure(game, moveFailure));
				return 1;
			} else {
				game.out().printf("%s does not have that item.%n", aftik.getName());
				return 0;
			}
		} else {
			game.out().printf("There is no such aftik in the room.%n");
			return 0;
		}
	}
	
	private static int searchForAndIfNotBlocked(GameInstance game, Aftik aftik, ObjectType type, Consumer<Item> onSuccess, Runnable onNoMatch) {
		Optional<Item> optionalItem = aftik.findNearest(Item.CAST.filter(type::matching));
		if (optionalItem.isPresent()) {
			Item item = optionalItem.get();
			
			Optional<GameObject> blocking = aftik.findBlockingTo(item.getCoord());
			if (blocking.isEmpty()) {
				onSuccess.accept(item);
				return 1;
			} else {
				ActionHandler.printBlocking(game, blocking.get());
				return 0;
			}
		} else {
			onNoMatch.run();
			return 0;
		}
	}
}