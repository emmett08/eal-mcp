#![allow(nonstandard_style)]
// Generated from grammar/EAL.g4 by ANTLR 4.13.2
use antlr4rust::tree::ParseTreeListener;
use super::ealparser::*;

pub trait EALListener<'input> : ParseTreeListener<'input,EALParserContextType>{
/**
 * Enter a parse tree produced by {@link EALParser#program}.
 * @param ctx the parse tree
 */
fn enter_program(&mut self, _ctx: &ProgramContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#program}.
 * @param ctx the parse tree
 */
fn exit_program(&mut self, _ctx: &ProgramContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#declaration}.
 * @param ctx the parse tree
 */
fn enter_declaration(&mut self, _ctx: &DeclarationContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#declaration}.
 * @param ctx the parse tree
 */
fn exit_declaration(&mut self, _ctx: &DeclarationContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#moduleDecl}.
 * @param ctx the parse tree
 */
fn enter_moduleDecl(&mut self, _ctx: &ModuleDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#moduleDecl}.
 * @param ctx the parse tree
 */
fn exit_moduleDecl(&mut self, _ctx: &ModuleDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#importDecl}.
 * @param ctx the parse tree
 */
fn enter_importDecl(&mut self, _ctx: &ImportDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#importDecl}.
 * @param ctx the parse tree
 */
fn exit_importDecl(&mut self, _ctx: &ImportDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#contextDecl}.
 * @param ctx the parse tree
 */
fn enter_contextDecl(&mut self, _ctx: &ContextDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#contextDecl}.
 * @param ctx the parse tree
 */
fn exit_contextDecl(&mut self, _ctx: &ContextDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#contextAttribute}.
 * @param ctx the parse tree
 */
fn enter_contextAttribute(&mut self, _ctx: &ContextAttributeContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#contextAttribute}.
 * @param ctx the parse tree
 */
fn exit_contextAttribute(&mut self, _ctx: &ContextAttributeContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#environmentDecl}.
 * @param ctx the parse tree
 */
fn enter_environmentDecl(&mut self, _ctx: &EnvironmentDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#environmentDecl}.
 * @param ctx the parse tree
 */
fn exit_environmentDecl(&mut self, _ctx: &EnvironmentDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#toolDecl}.
 * @param ctx the parse tree
 */
fn enter_toolDecl(&mut self, _ctx: &ToolDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#toolDecl}.
 * @param ctx the parse tree
 */
fn exit_toolDecl(&mut self, _ctx: &ToolDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#versionField}.
 * @param ctx the parse tree
 */
fn enter_versionField(&mut self, _ctx: &VersionFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#versionField}.
 * @param ctx the parse tree
 */
fn exit_versionField(&mut self, _ctx: &VersionFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#evidenceDecl}.
 * @param ctx the parse tree
 */
fn enter_evidenceDecl(&mut self, _ctx: &EvidenceDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#evidenceDecl}.
 * @param ctx the parse tree
 */
fn exit_evidenceDecl(&mut self, _ctx: &EvidenceDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#evidenceField}.
 * @param ctx the parse tree
 */
fn enter_evidenceField(&mut self, _ctx: &EvidenceFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#evidenceField}.
 * @param ctx the parse tree
 */
fn exit_evidenceField(&mut self, _ctx: &EvidenceFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#toolField}.
 * @param ctx the parse tree
 */
fn enter_toolField(&mut self, _ctx: &ToolFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#toolField}.
 * @param ctx the parse tree
 */
fn exit_toolField(&mut self, _ctx: &ToolFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#kindField}.
 * @param ctx the parse tree
 */
fn enter_kindField(&mut self, _ctx: &KindFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#kindField}.
 * @param ctx the parse tree
 */
fn exit_kindField(&mut self, _ctx: &KindFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#environmentField}.
 * @param ctx the parse tree
 */
fn enter_environmentField(&mut self, _ctx: &EnvironmentFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#environmentField}.
 * @param ctx the parse tree
 */
fn exit_environmentField(&mut self, _ctx: &EnvironmentFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#maxAgeField}.
 * @param ctx the parse tree
 */
fn enter_maxAgeField(&mut self, _ctx: &MaxAgeFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#maxAgeField}.
 * @param ctx the parse tree
 */
fn exit_maxAgeField(&mut self, _ctx: &MaxAgeFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#inputField}.
 * @param ctx the parse tree
 */
fn enter_inputField(&mut self, _ctx: &InputFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#inputField}.
 * @param ctx the parse tree
 */
fn exit_inputField(&mut self, _ctx: &InputFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#assumptionDecl}.
 * @param ctx the parse tree
 */
fn enter_assumptionDecl(&mut self, _ctx: &AssumptionDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#assumptionDecl}.
 * @param ctx the parse tree
 */
fn exit_assumptionDecl(&mut self, _ctx: &AssumptionDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#assumptionField}.
 * @param ctx the parse tree
 */
fn enter_assumptionField(&mut self, _ctx: &AssumptionFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#assumptionField}.
 * @param ctx the parse tree
 */
fn exit_assumptionField(&mut self, _ctx: &AssumptionFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#statementField}.
 * @param ctx the parse tree
 */
fn enter_statementField(&mut self, _ctx: &StatementFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#statementField}.
 * @param ctx the parse tree
 */
fn exit_statementField(&mut self, _ctx: &StatementFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#validateField}.
 * @param ctx the parse tree
 */
fn enter_validateField(&mut self, _ctx: &ValidateFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#validateField}.
 * @param ctx the parse tree
 */
fn exit_validateField(&mut self, _ctx: &ValidateFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#validFromField}.
 * @param ctx the parse tree
 */
fn enter_validFromField(&mut self, _ctx: &ValidFromFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#validFromField}.
 * @param ctx the parse tree
 */
fn exit_validFromField(&mut self, _ctx: &ValidFromFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#validUntilField}.
 * @param ctx the parse tree
 */
fn enter_validUntilField(&mut self, _ctx: &ValidUntilFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#validUntilField}.
 * @param ctx the parse tree
 */
fn exit_validUntilField(&mut self, _ctx: &ValidUntilFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#reasoningDecl}.
 * @param ctx the parse tree
 */
fn enter_reasoningDecl(&mut self, _ctx: &ReasoningDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#reasoningDecl}.
 * @param ctx the parse tree
 */
fn exit_reasoningDecl(&mut self, _ctx: &ReasoningDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#reasoningField}.
 * @param ctx the parse tree
 */
fn enter_reasoningField(&mut self, _ctx: &ReasoningFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#reasoningField}.
 * @param ctx the parse tree
 */
fn exit_reasoningField(&mut self, _ctx: &ReasoningFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#methodField}.
 * @param ctx the parse tree
 */
fn enter_methodField(&mut self, _ctx: &MethodFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#methodField}.
 * @param ctx the parse tree
 */
fn exit_methodField(&mut self, _ctx: &MethodFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#rationaleField}.
 * @param ctx the parse tree
 */
fn enter_rationaleField(&mut self, _ctx: &RationaleFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#rationaleField}.
 * @param ctx the parse tree
 */
fn exit_rationaleField(&mut self, _ctx: &RationaleFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#backingField}.
 * @param ctx the parse tree
 */
fn enter_backingField(&mut self, _ctx: &BackingFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#backingField}.
 * @param ctx the parse tree
 */
fn exit_backingField(&mut self, _ctx: &BackingFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#transferField}.
 * @param ctx the parse tree
 */
fn enter_transferField(&mut self, _ctx: &TransferFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#transferField}.
 * @param ctx the parse tree
 */
fn exit_transferField(&mut self, _ctx: &TransferFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#claimDecl}.
 * @param ctx the parse tree
 */
fn enter_claimDecl(&mut self, _ctx: &ClaimDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#claimDecl}.
 * @param ctx the parse tree
 */
fn exit_claimDecl(&mut self, _ctx: &ClaimDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#claimField}.
 * @param ctx the parse tree
 */
fn enter_claimField(&mut self, _ctx: &ClaimFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#claimField}.
 * @param ctx the parse tree
 */
fn exit_claimField(&mut self, _ctx: &ClaimFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#propositionDecl}.
 * @param ctx the parse tree
 */
fn enter_propositionDecl(&mut self, _ctx: &PropositionDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#propositionDecl}.
 * @param ctx the parse tree
 */
fn exit_propositionDecl(&mut self, _ctx: &PropositionDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#propositionField}.
 * @param ctx the parse tree
 */
fn enter_propositionField(&mut self, _ctx: &PropositionFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#propositionField}.
 * @param ctx the parse tree
 */
fn exit_propositionField(&mut self, _ctx: &PropositionFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#subjectField}.
 * @param ctx the parse tree
 */
fn enter_subjectField(&mut self, _ctx: &SubjectFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#subjectField}.
 * @param ctx the parse tree
 */
fn exit_subjectField(&mut self, _ctx: &SubjectFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#quantityField}.
 * @param ctx the parse tree
 */
fn enter_quantityField(&mut self, _ctx: &QuantityFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#quantityField}.
 * @param ctx the parse tree
 */
fn exit_quantityField(&mut self, _ctx: &QuantityFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#unitField}.
 * @param ctx the parse tree
 */
fn enter_unitField(&mut self, _ctx: &UnitFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#unitField}.
 * @param ctx the parse tree
 */
fn exit_unitField(&mut self, _ctx: &UnitFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#scopeField}.
 * @param ctx the parse tree
 */
fn enter_scopeField(&mut self, _ctx: &ScopeFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#scopeField}.
 * @param ctx the parse tree
 */
fn exit_scopeField(&mut self, _ctx: &ScopeFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#queryField}.
 * @param ctx the parse tree
 */
fn enter_queryField(&mut self, _ctx: &QueryFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#queryField}.
 * @param ctx the parse tree
 */
fn exit_queryField(&mut self, _ctx: &QueryFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#resultField}.
 * @param ctx the parse tree
 */
fn enter_resultField(&mut self, _ctx: &ResultFieldContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#resultField}.
 * @param ctx the parse tree
 */
fn exit_resultField(&mut self, _ctx: &ResultFieldContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#argumentDecl}.
 * @param ctx the parse tree
 */
fn enter_argumentDecl(&mut self, _ctx: &ArgumentDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#argumentDecl}.
 * @param ctx the parse tree
 */
fn exit_argumentDecl(&mut self, _ctx: &ArgumentDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#argumentFlow}.
 * @param ctx the parse tree
 */
fn enter_argumentFlow(&mut self, _ctx: &ArgumentFlowContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#argumentFlow}.
 * @param ctx the parse tree
 */
fn exit_argumentFlow(&mut self, _ctx: &ArgumentFlowContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#support}.
 * @param ctx the parse tree
 */
fn enter_support(&mut self, _ctx: &SupportContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#support}.
 * @param ctx the parse tree
 */
fn exit_support(&mut self, _ctx: &SupportContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#supportGroup}.
 * @param ctx the parse tree
 */
fn enter_supportGroup(&mut self, _ctx: &SupportGroupContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#supportGroup}.
 * @param ctx the parse tree
 */
fn exit_supportGroup(&mut self, _ctx: &SupportGroupContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#supportKind}.
 * @param ctx the parse tree
 */
fn enter_supportKind(&mut self, _ctx: &SupportKindContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#supportKind}.
 * @param ctx the parse tree
 */
fn exit_supportKind(&mut self, _ctx: &SupportKindContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#patternDecl}.
 * @param ctx the parse tree
 */
fn enter_patternDecl(&mut self, _ctx: &PatternDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#patternDecl}.
 * @param ctx the parse tree
 */
fn exit_patternDecl(&mut self, _ctx: &PatternDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#patternParameter}.
 * @param ctx the parse tree
 */
fn enter_patternParameter(&mut self, _ctx: &PatternParameterContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#patternParameter}.
 * @param ctx the parse tree
 */
fn exit_patternParameter(&mut self, _ctx: &PatternParameterContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#parameterKind}.
 * @param ctx the parse tree
 */
fn enter_parameterKind(&mut self, _ctx: &ParameterKindContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#parameterKind}.
 * @param ctx the parse tree
 */
fn exit_parameterKind(&mut self, _ctx: &ParameterKindContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#applicationDecl}.
 * @param ctx the parse tree
 */
fn enter_applicationDecl(&mut self, _ctx: &ApplicationDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#applicationDecl}.
 * @param ctx the parse tree
 */
fn exit_applicationDecl(&mut self, _ctx: &ApplicationDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#patternBinding}.
 * @param ctx the parse tree
 */
fn enter_patternBinding(&mut self, _ctx: &PatternBindingContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#patternBinding}.
 * @param ctx the parse tree
 */
fn exit_patternBinding(&mut self, _ctx: &PatternBindingContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#bindingValue}.
 * @param ctx the parse tree
 */
fn enter_bindingValue(&mut self, _ctx: &BindingValueContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#bindingValue}.
 * @param ctx the parse tree
 */
fn exit_bindingValue(&mut self, _ctx: &BindingValueContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#patternGuard}.
 * @param ctx the parse tree
 */
fn enter_patternGuard(&mut self, _ctx: &PatternGuardContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#patternGuard}.
 * @param ctx the parse tree
 */
fn exit_patternGuard(&mut self, _ctx: &PatternGuardContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#objectionDecl}.
 * @param ctx the parse tree
 */
fn enter_objectionDecl(&mut self, _ctx: &ObjectionDeclContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#objectionDecl}.
 * @param ctx the parse tree
 */
fn exit_objectionDecl(&mut self, _ctx: &ObjectionDeclContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#objectionSupport}.
 * @param ctx the parse tree
 */
fn enter_objectionSupport(&mut self, _ctx: &ObjectionSupportContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#objectionSupport}.
 * @param ctx the parse tree
 */
fn exit_objectionSupport(&mut self, _ctx: &ObjectionSupportContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#objectionGroup}.
 * @param ctx the parse tree
 */
fn enter_objectionGroup(&mut self, _ctx: &ObjectionGroupContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#objectionGroup}.
 * @param ctx the parse tree
 */
fn exit_objectionGroup(&mut self, _ctx: &ObjectionGroupContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#objectionSupportKind}.
 * @param ctx the parse tree
 */
fn enter_objectionSupportKind(&mut self, _ctx: &ObjectionSupportKindContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#objectionSupportKind}.
 * @param ctx the parse tree
 */
fn exit_objectionSupportKind(&mut self, _ctx: &ObjectionSupportKindContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#targetKind}.
 * @param ctx the parse tree
 */
fn enter_targetKind(&mut self, _ctx: &TargetKindContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#targetKind}.
 * @param ctx the parse tree
 */
fn exit_targetKind(&mut self, _ctx: &TargetKindContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#argumentationDirective}.
 * @param ctx the parse tree
 */
fn enter_argumentationDirective(&mut self, _ctx: &ArgumentationDirectiveContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#argumentationDirective}.
 * @param ctx the parse tree
 */
fn exit_argumentationDirective(&mut self, _ctx: &ArgumentationDirectiveContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#referenceList}.
 * @param ctx the parse tree
 */
fn enter_referenceList(&mut self, _ctx: &ReferenceListContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#referenceList}.
 * @param ctx the parse tree
 */
fn exit_referenceList(&mut self, _ctx: &ReferenceListContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#reference}.
 * @param ctx the parse tree
 */
fn enter_reference(&mut self, _ctx: &ReferenceContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#reference}.
 * @param ctx the parse tree
 */
fn exit_reference(&mut self, _ctx: &ReferenceContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#qualifiedName}.
 * @param ctx the parse tree
 */
fn enter_qualifiedName(&mut self, _ctx: &QualifiedNameContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#qualifiedName}.
 * @param ctx the parse tree
 */
fn exit_qualifiedName(&mut self, _ctx: &QualifiedNameContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#predicate}.
 * @param ctx the parse tree
 */
fn enter_predicate(&mut self, _ctx: &PredicateContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#predicate}.
 * @param ctx the parse tree
 */
fn exit_predicate(&mut self, _ctx: &PredicateContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#expression}.
 * @param ctx the parse tree
 */
fn enter_expression(&mut self, _ctx: &ExpressionContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#expression}.
 * @param ctx the parse tree
 */
fn exit_expression(&mut self, _ctx: &ExpressionContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#orExpression}.
 * @param ctx the parse tree
 */
fn enter_orExpression(&mut self, _ctx: &OrExpressionContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#orExpression}.
 * @param ctx the parse tree
 */
fn exit_orExpression(&mut self, _ctx: &OrExpressionContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#andExpression}.
 * @param ctx the parse tree
 */
fn enter_andExpression(&mut self, _ctx: &AndExpressionContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#andExpression}.
 * @param ctx the parse tree
 */
fn exit_andExpression(&mut self, _ctx: &AndExpressionContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#comparisonExpression}.
 * @param ctx the parse tree
 */
fn enter_comparisonExpression(&mut self, _ctx: &ComparisonExpressionContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#comparisonExpression}.
 * @param ctx the parse tree
 */
fn exit_comparisonExpression(&mut self, _ctx: &ComparisonExpressionContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#additiveExpression}.
 * @param ctx the parse tree
 */
fn enter_additiveExpression(&mut self, _ctx: &AdditiveExpressionContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#additiveExpression}.
 * @param ctx the parse tree
 */
fn exit_additiveExpression(&mut self, _ctx: &AdditiveExpressionContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#multiplicativeExpression}.
 * @param ctx the parse tree
 */
fn enter_multiplicativeExpression(&mut self, _ctx: &MultiplicativeExpressionContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#multiplicativeExpression}.
 * @param ctx the parse tree
 */
fn exit_multiplicativeExpression(&mut self, _ctx: &MultiplicativeExpressionContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#unaryExpression}.
 * @param ctx the parse tree
 */
fn enter_unaryExpression(&mut self, _ctx: &UnaryExpressionContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#unaryExpression}.
 * @param ctx the parse tree
 */
fn exit_unaryExpression(&mut self, _ctx: &UnaryExpressionContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#primaryExpression}.
 * @param ctx the parse tree
 */
fn enter_primaryExpression(&mut self, _ctx: &PrimaryExpressionContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#primaryExpression}.
 * @param ctx the parse tree
 */
fn exit_primaryExpression(&mut self, _ctx: &PrimaryExpressionContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#comparator}.
 * @param ctx the parse tree
 */
fn enter_comparator(&mut self, _ctx: &ComparatorContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#comparator}.
 * @param ctx the parse tree
 */
fn exit_comparator(&mut self, _ctx: &ComparatorContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#jsonValue}.
 * @param ctx the parse tree
 */
fn enter_jsonValue(&mut self, _ctx: &JsonValueContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#jsonValue}.
 * @param ctx the parse tree
 */
fn exit_jsonValue(&mut self, _ctx: &JsonValueContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#jsonObject}.
 * @param ctx the parse tree
 */
fn enter_jsonObject(&mut self, _ctx: &JsonObjectContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#jsonObject}.
 * @param ctx the parse tree
 */
fn exit_jsonObject(&mut self, _ctx: &JsonObjectContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#jsonArray}.
 * @param ctx the parse tree
 */
fn enter_jsonArray(&mut self, _ctx: &JsonArrayContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#jsonArray}.
 * @param ctx the parse tree
 */
fn exit_jsonArray(&mut self, _ctx: &JsonArrayContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#jsonScalar}.
 * @param ctx the parse tree
 */
fn enter_jsonScalar(&mut self, _ctx: &JsonScalarContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#jsonScalar}.
 * @param ctx the parse tree
 */
fn exit_jsonScalar(&mut self, _ctx: &JsonScalarContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#signedNumber}.
 * @param ctx the parse tree
 */
fn enter_signedNumber(&mut self, _ctx: &SignedNumberContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#signedNumber}.
 * @param ctx the parse tree
 */
fn exit_signedNumber(&mut self, _ctx: &SignedNumberContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#identifier}.
 * @param ctx the parse tree
 */
fn enter_identifier(&mut self, _ctx: &IdentifierContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#identifier}.
 * @param ctx the parse tree
 */
fn exit_identifier(&mut self, _ctx: &IdentifierContext<'input>) { }
/**
 * Enter a parse tree produced by {@link EALParser#lineEnd}.
 * @param ctx the parse tree
 */
fn enter_lineEnd(&mut self, _ctx: &LineEndContext<'input>) { }
/**
 * Exit a parse tree produced by {@link EALParser#lineEnd}.
 * @param ctx the parse tree
 */
fn exit_lineEnd(&mut self, _ctx: &LineEndContext<'input>) { }

}

antlr4rust::coerce_from!{ 'input : EALListener<'input> }


