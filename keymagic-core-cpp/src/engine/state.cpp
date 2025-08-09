#include <keymagic/engine.h>
#include <algorithm>

namespace keymagic {

EngineState::EngineState() {
}

EngineState::~EngineState() {
}

const std::u16string& EngineState::getComposingText() const {
    return composingText_;
}

void EngineState::setComposingText(const std::u16string& text) {
    composingText_ = text;
}

void EngineState::appendToComposingText(const std::u16string& text) {
    composingText_ += text;
}

void EngineState::clearComposingText() {
    composingText_.clear();
}

const std::unordered_set<int>& EngineState::getActiveStates() const {
    return activeStates_;
}

void EngineState::setActiveStates(const std::unordered_set<int>& states) {
    activeStates_ = states;
}

void EngineState::addActiveState(int stateId) {
    activeStates_.insert(stateId);
}

void EngineState::removeActiveState(int stateId) {
    activeStates_.erase(stateId);
}

void EngineState::clearActiveStates() {
    activeStates_.clear();
}

bool EngineState::hasActiveState(int stateId) const {
    return activeStates_.find(stateId) != activeStates_.end();
}

std::u16string EngineState::getContext(size_t maxLength) const {
    if (composingText_.size() <= maxLength) {
        return composingText_;
    }
    
    // Return last maxLength characters
    return composingText_.substr(composingText_.size() - maxLength);
}

void EngineState::reset() {
    composingText_.clear();
    activeStates_.clear();
}

std::unique_ptr<EngineState> EngineState::clone() const {
    auto copy = std::make_unique<EngineState>();
    copy->composingText_ = composingText_;
    copy->activeStates_ = activeStates_;
    return copy;
}

void EngineState::copyFrom(const EngineState& other) {
    composingText_ = other.composingText_;
    activeStates_ = other.activeStates_;
}

} // namespace keymagic