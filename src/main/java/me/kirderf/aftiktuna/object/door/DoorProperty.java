package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

import java.util.List;

public class DoorProperty {
	public static final DoorProperty STUCK = new DoorProperty(FailureType.STUCK, Status.NEED_TOOL);
	public static final DoorProperty SEALED = new DoorProperty(FailureType.SEALED, Status.NEED_BREAK_TOOL);
	public static final DoorProperty EMPTY = new DoorProperty(null, Status.NOT_STUCK);
	
	private final FailureType entryFailure;
	private final Status forceStatus;
	
	protected DoorProperty(FailureType entryFailure, Status forceStatus) {
		this.entryFailure = entryFailure;
		this.forceStatus = forceStatus;
	}
	
	public EnterResult checkEntry(Aftik aftik) {
		if (entryFailure == null)
			return new EnterResult();
		else
			return new EnterResult(entryFailure);
	}
	
	public final ForceResult.PropertyResult tryForce(Aftik aftik) {
		for (Method method : forceStatus.getAvailableMethods()) {
			if(aftik.hasItem(method.tool())) {
				return ForceResult.success(method);
			}
		}
		return ForceResult.status(forceStatus);
	}
	
	public record FailureType(String adjective) {
		public static final FailureType STUCK = new FailureType("stuck");
		public static final FailureType LOCKED = new FailureType("locked");
		public static final FailureType SEALED = new FailureType("sealed shut");
	}
	
	public record Method(ItemType tool, String text) {
		public static final Method FORCE = new Method(ObjectTypes.CROWBAR, "forced open");
		public static final Method CUT = new Method(ObjectTypes.BLOWTORCH, "cut open");
	}
	
	public enum Status {
		NEED_TOOL(Method.FORCE, Method.CUT),
		NEED_BREAK_TOOL(Method.CUT),
		NOT_STUCK;
		
		private final List<Method> availableMethods;
		
		Status(Method... availableMethods) {
			this.availableMethods = List.of(availableMethods);
		}
		
		public List<Method> getAvailableMethods() {
			return availableMethods;
		}
	}
}