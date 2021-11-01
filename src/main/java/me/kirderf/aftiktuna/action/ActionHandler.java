package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.ObjectArgument;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.entity.Aftik;
import me.kirderf.aftiktuna.level.object.entity.AttackResult;
import me.kirderf.aftiktuna.level.object.entity.Creature;
import me.kirderf.aftiktuna.level.object.entity.Entity;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;

public final class ActionHandler {
	private final CommandDispatcher<GameInstance> dispatcher = new CommandDispatcher<>();
	
	public ActionHandler() {
		ItemActions.register(dispatcher);
		DoorActions.register(dispatcher);
		dispatcher.register(literal("attack").then(argument("creature", ObjectArgument.create(ObjectTypes.CREATURES))
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
			
			Entity.MoveAndAttackResult result = aftik.moveAndAttack(creature);
			
			result.either().run(ActionHandler::printAttackAction, ActionHandler::printMoveFailure);
			
			return result.success() ? 1 : 0;
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
	
	private static void printAttackAction(AttackResult result) {
		String name = result.attacked().getType().lowerCaseName();
		switch(result.type()) {
			case HIT -> System.out.printf("You attacked the %s.%n", name);
			case KILL -> System.out.printf("You attacked and killed the %s.%n", name);
			case DODGE -> System.out.printf("The %s dodged your attack.%n", name);
		}
	}
	
	public void handleCreatures(GameInstance game) {
		Room room = game.getAftik().getRoom();
		room.objectStream().flatMap(Creature.CAST.toStream()).filter(Entity::isAlive).forEach(creature -> handleCreature(game, creature));
	}
	
	private static void handleCreature(GameInstance game, Creature creature) {
		Optional<AttackResult> result = creature.doAction(game.getAftik());
		result.ifPresent(attack -> {
			String name = creature.getType().lowerCaseName();
			switch(attack.type()) {
				case HIT -> System.out.printf("The %s attacked you.%n", name);
				case KILL -> System.out.printf("The %s attacked and killed you.%n", name);
				case DODGE -> System.out.printf("You dodged the %s's attack.%n", name);
			}
		});
	}
}
