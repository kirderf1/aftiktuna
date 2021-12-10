package me.kirderf.aftiktuna.command.game;

import me.kirderf.aftiktuna.command.CommandContext;
import me.kirderf.aftiktuna.command.CommandUtil;
import me.kirderf.aftiktuna.object.ObjectArgument;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorType;
import me.kirderf.aftiktuna.object.entity.Aftik;

public final class DoorCommands {
	static void register() {
		GameCommands.DISPATCHER.register(GameCommands.literal("enter").then(GameCommands.argument("door", ObjectArgument.create(ObjectTypes.DOORS))
				.executes(context -> enterDoor(context.getSource(), ObjectArgument.getType(context, "door", DoorType.class)))));
		GameCommands.DISPATCHER.register(GameCommands.literal("force").then(GameCommands.argument("door", ObjectArgument.create(ObjectTypes.FORCEABLE))
				.executes(context -> forceDoor(context.getSource(), ObjectArgument.getType(context, "door", DoorType.class)))));
		GameCommands.DISPATCHER.register(GameCommands.literal("enter").then(GameCommands.literal("ship")
				.executes(context -> enterDoor(context.getSource(), ObjectTypes.SHIP_ENTRANCE))));
		GameCommands.DISPATCHER.register(GameCommands.literal("exit").then(GameCommands.literal("ship")
				.executes(context -> enterDoor(context.getSource(), ObjectTypes.SHIP_EXIT))));
	}
	
	private static int enterDoor(CommandContext context, DoorType doorType) {
		Aftik aftik = context.getControlledAftik();
		return CommandUtil.searchForAccessible(context, aftik, Door.CAST.filter(doorType::matching), true,
				door -> context.action(out -> aftik.moveAndEnter(door, out)),
				() -> context.printNoAction("There is no such %s here to go through.", doorType.getCategoryName()));
	}
	
	private static int forceDoor(CommandContext context, DoorType doorType) {
		Aftik aftik = context.getControlledAftik();
		return CommandUtil.searchForAccessible(context, aftik, Door.CAST.filter(doorType::matching), true,
				door -> context.action(out -> aftik.moveAndForce(door, out)),
				() -> context.printNoAction("There is no such %s here.", doorType.getCategoryName()));
	}
}