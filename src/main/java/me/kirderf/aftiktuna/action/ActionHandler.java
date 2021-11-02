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
			game.out().printf("Unexpected input \"%s\"%n", input);
			return 0;
		}
	}
	
	private static int attack(GameInstance game, ObjectType creatureType) {
		Aftik aftik = game.getAftik();
		
		Optional<Creature> optionalCreature = aftik.findNearest(OptionalFunction.of(creatureType::matching).flatMap(Creature.CAST));
		if (optionalCreature.isPresent()) {
			Creature creature = optionalCreature.get();
			
			Entity.MoveAndAttackResult result = aftik.moveAndAttack(creature);
			
			result.either().run(attack -> printAttackAction(game, attack), move -> printMoveFailure(game, move));
			
			return result.success() ? 1 : 0;
		} else {
			game.out().println("There is no such creature to attack.");
			return 0;
		}
	}
	
	static void printMoveFailure(GameInstance game, Entity.MoveFailure result) {
		game.out().printf("The %s is blocking the way.%n", result.blocking().getType().lowerCaseName());
	}
	
	private static void printAttackAction(GameInstance game, AttackResult result) {
		String name = result.attacked().getType().lowerCaseName();
		switch(result.type()) {
			case DIRECT_HIT -> game.out().printf("You got a direct hit on%s the %s.%n", result.isKill() ? " and killed" : "", name);
			case GRAZING_HIT -> game.out().printf("You grazed%s the %s.%n", result.isKill() ? " and killed" : "", name);
			case DODGE -> game.out().printf("The %s dodged your attack.%n", name);
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
				case DIRECT_HIT -> game.out().printf("The %s's attack hit you directly%s.%n", name, attack.isKill() ? " and killed you" : "");
				case GRAZING_HIT -> game.out().printf("The %s's attack grazed%s you.%n", name, attack.isKill() ? " and killed" : "");
				case DODGE -> game.out().printf("You dodged the %s's attack.%n", name);
			}
		});
	}
}
