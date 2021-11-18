package me.kirderf.aftiktuna.action;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.action.result.AttackResult;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Position;
import me.kirderf.aftiktuna.location.Ship;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.AftikNPC;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.object.entity.Entity;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;
import java.util.function.IntSupplier;
import java.util.function.ToIntFunction;
import java.util.stream.Collectors;

public final class ActionHandler {
	private final CommandDispatcher<InputActionContext> dispatcher = new CommandDispatcher<>();
	
	public ActionHandler() {
		ItemActions.register(dispatcher);
		DoorActions.register(dispatcher);
		dispatcher.register(literal("attack").then(argument("creature", ObjectArgument.create(ObjectTypes.CREATURES))
				.executes(context -> attack(context.getSource(), ObjectArgument.getType(context, "creature")))));
		dispatcher.register(literal("launch").then(literal("ship").executes(context -> launchShip(context.getSource()))));
		dispatcher.register(literal("control").then(argument("name", StringArgumentType.string())
				.executes(context -> controlAftik(context.getSource(), StringArgumentType.getString(context, "name")))));
		dispatcher.register(literal("recruit").then(literal("aftik").executes(context -> recruitAftik(context.getSource()))));
		dispatcher.register(literal("wait").executes(context -> context.getSource().action()));
		dispatcher.register(literal("status").executes(context -> printStatus(context.getSource())));
		dispatcher.register(literal("help").executes(context -> printCommands(context.getSource())));
	}
	
	static LiteralArgumentBuilder<InputActionContext> literal(String str) {
		return LiteralArgumentBuilder.literal(str);
	}
	
	static <T> RequiredArgumentBuilder<InputActionContext, T> argument(String name, ArgumentType<T> argumentType) {
		return RequiredArgumentBuilder.argument(name, argumentType);
	}
	
	public int handleInput(InputActionContext context, String input) {
		try {
			return dispatcher.execute(input, context);
		} catch(CommandSyntaxException ignored) {
			return context.printNoAction("Unexpected input \"%s\"%n", input);
		}
	}
	
	private int printStatus(InputActionContext context) {
		return context.noAction(out -> context.getGame().getStatusPrinter().printCrewStatus());
	}
	
	private int printCommands(InputActionContext context) {
		return context.noAction(out -> {
			out.printf("Commands:%n");
			
			for (String command : dispatcher.getSmartUsage(dispatcher.getRoot(), context).values()) {
				out.printf("> %s%n", command);
			}
		});
	}
	
	private static int attack(InputActionContext context, ObjectType creatureType) {
		Aftik aftik = context.getControlledAftik();
		
		Optional<Creature> optionalCreature = aftik.findNearest(OptionalFunction.of(creatureType::matching).flatMap(Creature.CAST), false);
		
		return optionalCreature.map(creature -> context.action(out -> aftik.moveAndAttack(creature, out)))
				.orElseGet(() -> context.printNoAction("There is no such creature to attack."));
	}
	
	private static int launchShip(InputActionContext context) {
		Aftik aftik = context.getControlledAftik();
		
		if (aftik.hasItem(ObjectTypes.FUEL_CAN)) {
			if (isNearShip(aftik, context.getCrew().getShip())) {
				
				return context.action(aftik.getMind()::setLaunchShip);
			} else {
				return context.printNoAction("%s need to be near the ship in order to launch it.%n", aftik.getName());
			}
		} else {
			return context.printNoAction("%s need a fuel can to launch the ship.%n", aftik.getName());
		}
	}
	
	private static boolean isNearShip(Aftik aftik, Ship ship) {
		return aftik.getArea() == ship.getRoom() || aftik.isAnyNear(ObjectTypes.SHIP_ENTRANCE::matching);
	}
	
	private static int controlAftik(InputActionContext context, String name) {
		Optional<Aftik> aftikOptional = context.getCrew().findByName(name);
		if (aftikOptional.isPresent()) {
			Aftik aftik = aftikOptional.get();
			if (aftik != context.getControlledAftik()) {
				return context.action(out -> {
					context.getGame().setControllingAftik(aftik);
					context.getGame().getStatusPrinter().printStatus(true);
				});
			} else {
				return context.printNoAction("You're already in control of them.%n");
			}
		} else {
			return context.printNoAction("There is no crew member by that name.%n");
		}
	}
	
	private static int recruitAftik(InputActionContext context) {
		Aftik aftik = context.getControlledAftik();
		Optional<AftikNPC> npcOptional = aftik.findNearest(AftikNPC.CAST.filter(ObjectTypes.AFTIK::matching), false);
		
		if (npcOptional.isPresent()) {
			AftikNPC npc = npcOptional.get();
			
			if (context.getCrew().hasCapacity()) {
				Position pos = npc.getPosition().getPosTowards(aftik.getCoord());
				return ifNotBlocked(context, aftik, pos, () -> context.action(out -> {
					boolean success = aftik.tryMoveNextTo(npc.getPosition(), out);
					
					if (success) {
						context.getGame().recruitAftik(npc);
					}
				}));
			} else {
				return context.printNoAction("There is not enough room for another crew member.%n");
			}
		} else {
			return context.printNoAction("There is no aftik here to recruit.%n");
		}
	}
	
	static <T extends GameObject> int searchForAndIfNotBlocked(InputActionContext context, Aftik aftik, OptionalFunction<GameObject, T> mapper, ToIntFunction<T> onSuccess, IntSupplier onNoMatch) {
		Optional<T> optionalDoor = aftik.findNearest(mapper, true);
		if (optionalDoor.isPresent()) {
			T object = optionalDoor.get();
			
			return ifNotBlocked(context, aftik, object.getPosition(), () -> onSuccess.applyAsInt(object));
		} else {
			return onNoMatch.getAsInt();
		}
	}
	
	static int ifNotBlocked(InputActionContext context, Aftik aftik, Position pos, IntSupplier onSuccess) {
		Optional<GameObject> blocking = aftik.findBlockingTo(pos.coord());
		if (blocking.isEmpty()) {
			return onSuccess.getAsInt();
		} else {
			return context.printNoAction(createBlockingMessage(blocking.get()) + "%n");
		}
	}
	
	public static String createBlockingMessage(GameObject blocking) {
		return "%s is blocking the way.".formatted(blocking.getDisplayName(true, true));
	}
	
	public static void printAttackAction(ActionPrinter out, Entity attacker, AttackResult result) {
		Entity attacked = result.attacked();
		switch(result.type()) {
			case DIRECT_HIT -> out.printAt(attacker, condition("%s got a direct hit on[ and killed] %s.", result.isKill()),
					attacker.getDisplayName(true, true), attacked.getDisplayName(true, false));
			case GRAZING_HIT -> out.printAt(attacker, condition("%s's attack grazed[ and killed] %s.", result.isKill()),
					attacker.getDisplayName(true, true), attacked.getDisplayName(true, false));
			case DODGE -> out.printAt(attacker, "%s dodged %s's attack.",
					attacked.getDisplayName(true, true), attacker.getDisplayName(true, false));
		}
	}
	
	public void handleEntities(GameInstance game, ActionPrinter out) {
		
		for (Entity entity : game.getGameObjectStream().flatMap(Entity.CAST.toStream()).collect(Collectors.toList())) {
			if (entity.isAlive() && entity != game.getCrew().getAftik()) {
				entity.performAction(out);
			}
		}
	}
	
	private static String condition(String text, boolean b) {
		if (b)
			return text.replaceAll("[\\[\\]]", "");
		else return text.replaceAll("\\[.*]", "");
	}
}
