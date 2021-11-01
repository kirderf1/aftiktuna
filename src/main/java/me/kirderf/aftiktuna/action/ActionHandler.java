package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.*;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;

public final class ActionHandler {
	private final CommandDispatcher<GameInstance> dispatcher = new CommandDispatcher<>();
	
	public ActionHandler() {
		ItemActions.register(dispatcher);
		DoorActions.register(dispatcher);
		dispatcher.register(literal("attack").then(argument("creature", ObjectArgument.create(ObjectType.CREATURES))
				.executes(context -> attack(context.getSource(), ObjectArgument.getType(context, "creature")))));
	}
	
	static LiteralArgumentBuilder<GameInstance> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	static <T> RequiredArgumentBuilder<GameInstance, T> argument(String name, ArgumentType<T> argumentType) {
		return RequiredArgumentBuilder.argument(name, argumentType);
	}
	
	public int handleInput(GameInstance game, String input) {
		try {
			return dispatcher.execute(input, game);
		} catch(CommandSyntaxException ignored) {
			System.out.printf("Unexpected input \"%s\"%n", input);
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
					
					System.out.printf("You attacked and killed the %s.%n", creatureType.lowerCaseName());
				} else {
					System.out.printf("You attacked the %s.%n", creatureType.lowerCaseName());
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
	
	static void printMoveFailure(Aftik.MoveResult result) {
		result.blocking().ifPresent(object ->
				System.out.printf("The %s is blocking the way.%n", object.getType().lowerCaseName())
		);
	}
	
	public void handleCreatures(GameInstance game) {
		Room room = game.getAftik().getRoom();
		room.objectStream().flatMap(Creature.CAST.toStream()).forEach(creature -> handleCreature(game, creature));
	}
	
	private static void handleCreature(GameInstance game, Creature creature) {
		if (!creature.isDead()) {
			Aftik aftik = game.getAftik();
			if (!aftik.isDead() && aftik.getPosition().isAdjacent(creature.getPosition())) {
				Entity.AttackResult result = aftik.receiveAttack(1);
				if (result.death()) {
					System.out.printf("The %s attacked and killed you.%n", creature.getType().lowerCaseName());
				} else {
					System.out.printf("The %s attacked you.%n", creature.getType().lowerCaseName());
				}
			}
		}
	}
}
