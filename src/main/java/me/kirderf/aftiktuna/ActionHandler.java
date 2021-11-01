package me.kirderf.aftiktuna;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.object.*;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.door.EnterResult;
import me.kirderf.aftiktuna.level.object.door.ForceResult;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Locale;
import java.util.Optional;

public final class ActionHandler {
	private static final CommandDispatcher<GameInstance> DISPATCHER = new CommandDispatcher<>();
	
	static {
		DISPATCHER.register(literal("take").then(argument("item", ObjectArgument.create(ObjectType.ITEMS))
				.executes(context -> takeItem(context.getSource(), ObjectArgument.getType(context, "item")))));
		DISPATCHER.register(literal("enter").then(argument("door", ObjectArgument.create(ObjectType.DOORS))
				.executes(context -> goThroughDoor(context.getSource(), ObjectArgument.getType(context, "door")))));
		DISPATCHER.register(literal("force").then(argument("door", ObjectArgument.create(ObjectType.DOORS))
				.executes(context -> forceDoor(context.getSource(), ObjectArgument.getType(context, "door")))));
		DISPATCHER.register(literal("attack").then(argument("creature", ObjectArgument.create(ObjectType.CREATURES))
				.executes(context -> attack(context.getSource(), ObjectArgument.getType(context, "creature")))));
		DISPATCHER.register(literal("wield").then(argument("item", ObjectArgument.create(ObjectType.WEAPONS))
				.executes(context -> wieldItem(context.getSource(), ObjectArgument.getType(context, "item")))));
	}
	
	private static LiteralArgumentBuilder<GameInstance> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	private static <T> RequiredArgumentBuilder<GameInstance, T> argument(String name, ArgumentType<T> argumentType) {
		return RequiredArgumentBuilder.argument(name, argumentType);
	}
	
	public static int handleInput(GameInstance game, String input) {
		try {
			return DISPATCHER.execute(input, game);
		} catch(CommandSyntaxException ignored) {
			System.out.printf("Unexpected input \"%s\"%n", input);
			return 0;
		}
	}
	
	private static int takeItem(GameInstance game, ObjectType type) {
		Aftik aftik = game.getAftik();
		Optional<GameObject> optionalItem = aftik.findNearest(OptionalFunction.of(GameObject::isItem).filter(type::matching));
		if(optionalItem.isPresent()) {
			
			GameObject item = optionalItem.get();
			Aftik.MoveResult move = aftik.tryMoveTo(item.getCoord());
			if (move.success()) {
				item.remove();
				aftik.addItem(type);
				
				System.out.printf("You picked up the %s.%n", type.name().toLowerCase(Locale.ROOT));
				return 1;
			} else {
				printMoveFailure(move);
				return 0;
			}
		} else {
			System.out.printf("There is no %s here to pick up.%n", type.name().toLowerCase(Locale.ROOT));
			return 0;
		}
	}
	
	private static int wieldItem(GameInstance game, ObjectType itemType) {
		Aftik aftik = game.getAftik();
		
		if (aftik.wieldFromInventory(itemType)) {
			System.out.printf("You wielded a %s.%n", itemType.name().toLowerCase(Locale.ROOT));
			return 1;
		} else {
			Optional<GameObject> optionalItem = aftik.findNearest(OptionalFunction.of(GameObject::isItem).filter(itemType::matching));
			if(optionalItem.isPresent()) {
				
				GameObject item = optionalItem.get();
				Aftik.MoveResult move = aftik.tryMoveTo(item.getCoord());
				if (move.success()) {
					item.remove();
					aftik.wield(itemType);
					
					System.out.printf("You picked up and wielded the %s.%n", itemType.name().toLowerCase(Locale.ROOT));
					return 1;
				} else {
					printMoveFailure(move);
					return 0;
				}
			} else {
				System.out.printf("There is no %s that you can wield.%n", itemType.name().toLowerCase(Locale.ROOT));
				return 0;
			}
		}
	}
	
	private static int goThroughDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		Optional<Door> optionalDoor = aftik.findNearest(OptionalFunction.of(doorType::matching).flatMap(Door.CAST));
		if(optionalDoor.isPresent()) {
			
			Aftik.MoveResult move = aftik.tryMoveTo(optionalDoor.get().getCoord());
			if (move.success()) {
				EnterResult result = optionalDoor.get().enter(aftik);
				
				printEnterResult(result);
				return 1;
			} else {
				printMoveFailure(move);
				return 0;
			}
		} else {
			System.out.println("There is no such door here to go through.");
			return 0;
		}
	}
	
	private static int forceDoor(GameInstance game, ObjectType doorType) {
		Aftik aftik = game.getAftik();
		Optional<Door> optionalDoor = aftik.findNearest(OptionalFunction.of(doorType::matching).flatMap(Door.CAST));
		if(optionalDoor.isPresent()) {
			
			Aftik.MoveResult move = aftik.tryMoveTo(optionalDoor.get().getCoord());
			if (move.success()) {
				ForceResult result = optionalDoor.get().force(aftik);
				
				printForceResult(result);
				return 1;
			} else {
				printMoveFailure(move);
				return 0;
			}
		} else {
			System.out.println("There is no such door here.");
			return 0;
		}
	}
	
	private static int attack(GameInstance game, ObjectType creatureType) {
		Aftik aftik = game.getAftik();
		
		Optional<Creature> optionalCreature = aftik.findNearest(OptionalFunction.of(creatureType::matching).flatMap(Creature.CAST));
		if (optionalCreature.isPresent()) {
			Creature creature = optionalCreature.get();
			
			Aftik.MoveResult move = aftik.tryMoveTo(creature.getPosition().getPosTowards(aftik.getCoord()).coord());
			if (move.success()) {
				Entity.AttackResult result = creature.receiveAttack(aftik.getAttackPower());
				if (result.death()) {
					creature.remove();
					
					System.out.printf("You attacked and killed the %s.%n", creatureType.name().toLowerCase(Locale.ROOT));
				} else {
					Entity.AttackResult retaliationResult = aftik.receiveAttack(1);
					
					if (retaliationResult.death()) {
						System.out.printf("You attacked the %s, which attacked back in retaliation, killing you.%n",
								creatureType.name().toLowerCase(Locale.ROOT));
					} else {
						System.out.printf("You attacked the %s, which attacked back in retaliation.%n", creatureType.name().toLowerCase(Locale.ROOT));
					}
				}
				return 1;
			} else {
				printMoveFailure(move);
				return 0;
			}
		} else {
			System.out.println("There is no such creature to attack.");
			return 0;
		}
	}
	
	private static void printMoveFailure(Aftik.MoveResult result) {
		result.blocking().ifPresent(object ->
				System.out.printf("The %s is blocking the way.%n", object.getType().name().toLowerCase(Locale.ROOT))
		);
	}
	
	private static void printEnterResult(EnterResult result) {
		result.either().run(ActionHandler::printEnterSuccess,
				failureType -> System.out.printf("The door is %s.%n", failureType.adjective()));
	}
	
	private static void printEnterSuccess(EnterResult.Success result) {
		result.usedItem().ifPresentOrElse(
				item -> System.out.printf("Using your %s, you entered the door into a new room.%n", item.name().toLowerCase(Locale.ROOT)),
				() -> System.out.printf("You entered the door into a new room.%n"));
	}
	
	private static void printForceResult(ForceResult result) {
		result.either().run(ActionHandler::printForceSuccess, ActionHandler::printForceStatus);
	}
	
	private static void printForceSuccess(ForceResult.Success result) {
		System.out.printf("You use your %s to %s.%n", result.item().name().toLowerCase(Locale.ROOT), result.method().text());
	}
	
	private static void printForceStatus(ForceResult.Status status) {
		switch(status) {
			case NOT_STUCK -> System.out.println("The door does not seem to be stuck.");
			case NEED_TOOL -> System.out.println("You need some sort of tool to force the door open.");
			case NEED_BREAK_TOOL -> System.out.println("You need some sort of tool to break the door open.");
		}
	}
}
