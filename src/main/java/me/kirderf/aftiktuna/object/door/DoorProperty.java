package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.action.result.ForceResult;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

import java.util.List;
import java.util.Optional;

/**
 * A property held by {@link DoorPairInfo} which determine the success of entry through the door,
 * and behaviors related to forcing the door.
 */
public final class DoorProperty {
	public static final DoorProperty STUCK = new DoorProperty(new EntryBlockingInfo("stuck"), ForceStatus.NEED_TOOL);
	public static final DoorProperty SEALED = new DoorProperty(new EntryBlockingInfo("sealed shut"), ForceStatus.NEED_BREAK_TOOL);
	public static final DoorProperty LOCKED = new DoorProperty(new EntryBlockingInfo("locked", ObjectTypes.KEYCARD), ForceStatus.NEED_BREAK_TOOL);
	public static final DoorProperty EMPTY = new DoorProperty(null, ForceStatus.NOT_STUCK);
	
	// If null, this property does not block entry. Otherwise, entry might be blocked depending on circumstances.
	private final EntryBlockingInfo entryInfo;
	private final ForceStatus forceStatus;
	
	private DoorProperty(EntryBlockingInfo entryInfo, ForceStatus forceStatus) {
		this.entryInfo = entryInfo;
		this.forceStatus = forceStatus;
	}
	
	public Optional<String> getAdjective() {
		return Optional.ofNullable(entryInfo).map(EntryBlockingInfo::adjective);
	}
	
	public List<Method> relevantForceMethods() {
		return forceStatus.getAvailableMethods();
	}
	
	public EnterResult checkEntry(Aftik aftik) {
		if (entryInfo == null)
			return EnterResult.success(this);
		else {
			if (aftik.hasItem(entryInfo.itemToPass)) {
				return EnterResult.success(this, entryInfo.itemToPass);
			} else {
				return EnterResult.failure(this, entryInfo.adjective);
			}
		}
	}
	
	public ForceResult.PropertyResult tryForce(Aftik aftik, ItemType item) {
		if (item != null) {
			if(canBeForcedWith(item.getForceMethod())) {
				return ForceResult.success(item);
			}
		} else {
			for (Method method : forceStatus.getAvailableMethods()) {
				Optional<ItemType> toolOptional = aftik.findItem(method::canBeUsedBy);
				if (toolOptional.isPresent()) {
					return ForceResult.success(toolOptional.get());
				}
			}
		}
		return ForceResult.status(forceStatus);
	}
	
	public boolean canBeForcedWith(Method method) {
		return forceStatus.getAvailableMethods().contains(method);
	}
	
	private record EntryBlockingInfo(String adjective, ItemType itemToPass) {
		private EntryBlockingInfo(String adjective) {
			this(adjective, null);
		}
	}
	
	public record Method(String text) {
		public static final Method FORCE = new Method("forced open");
		public static final Method CUT = new Method("cut open");
		
		public boolean canBeUsedBy(ItemType item) {
			return item.getForceMethod() == this;
		}
	}
	
	public enum ForceStatus {
		NEED_TOOL(Method.FORCE, Method.CUT),
		NEED_BREAK_TOOL(Method.CUT),
		NOT_STUCK;
		
		private final List<Method> availableMethods;
		
		ForceStatus(Method... availableMethods) {
			this.availableMethods = List.of(availableMethods);
		}
		
		public List<Method> getAvailableMethods() {
			return availableMethods;
		}
	}
}