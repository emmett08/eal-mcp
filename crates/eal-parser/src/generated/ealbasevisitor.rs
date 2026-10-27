
// Generated from grammar/EAL.g4 by ANTLR 4.13.2

use antlr4rust::tree::ParseTreeVisitor;
use super::ealparser::*;

// A complete Visitor for a parse tree produced by EALParser.

pub trait EALBaseVisitor<'input>:
    ParseTreeVisitor<'input, EALParserContextType> {
	// Visit a parse tree produced by EALParser#program.
	fn visit_program(&mut self, ctx: &ProgramContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#declaration.
	fn visit_declaration(&mut self, ctx: &DeclarationContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#moduleDecl.
	fn visit_moduledecl(&mut self, ctx: &ModuleDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#importDecl.
	fn visit_importdecl(&mut self, ctx: &ImportDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#contextDecl.
	fn visit_contextdecl(&mut self, ctx: &ContextDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#contextAttribute.
	fn visit_contextattribute(&mut self, ctx: &ContextAttributeContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#environmentDecl.
	fn visit_environmentdecl(&mut self, ctx: &EnvironmentDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#toolDecl.
	fn visit_tooldecl(&mut self, ctx: &ToolDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#versionField.
	fn visit_versionfield(&mut self, ctx: &VersionFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#evidenceDecl.
	fn visit_evidencedecl(&mut self, ctx: &EvidenceDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#evidenceField.
	fn visit_evidencefield(&mut self, ctx: &EvidenceFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#toolField.
	fn visit_toolfield(&mut self, ctx: &ToolFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#kindField.
	fn visit_kindfield(&mut self, ctx: &KindFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#environmentField.
	fn visit_environmentfield(&mut self, ctx: &EnvironmentFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#maxAgeField.
	fn visit_maxagefield(&mut self, ctx: &MaxAgeFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#inputField.
	fn visit_inputfield(&mut self, ctx: &InputFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#inputBindingField.
	fn visit_inputbindingfield(&mut self, ctx: &InputBindingFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#deriveField.
	fn visit_derivefield(&mut self, ctx: &DeriveFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#assumptionDecl.
	fn visit_assumptiondecl(&mut self, ctx: &AssumptionDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#assumptionField.
	fn visit_assumptionfield(&mut self, ctx: &AssumptionFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#statementField.
	fn visit_statementfield(&mut self, ctx: &StatementFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#validateField.
	fn visit_validatefield(&mut self, ctx: &ValidateFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#validFromField.
	fn visit_validfromfield(&mut self, ctx: &ValidFromFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#validUntilField.
	fn visit_validuntilfield(&mut self, ctx: &ValidUntilFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#reasoningDecl.
	fn visit_reasoningdecl(&mut self, ctx: &ReasoningDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#reasoningField.
	fn visit_reasoningfield(&mut self, ctx: &ReasoningFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#methodField.
	fn visit_methodfield(&mut self, ctx: &MethodFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#rationaleField.
	fn visit_rationalefield(&mut self, ctx: &RationaleFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#backingField.
	fn visit_backingfield(&mut self, ctx: &BackingFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#transferField.
	fn visit_transferfield(&mut self, ctx: &TransferFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#claimDecl.
	fn visit_claimdecl(&mut self, ctx: &ClaimDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#claimField.
	fn visit_claimfield(&mut self, ctx: &ClaimFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#propositionDecl.
	fn visit_propositiondecl(&mut self, ctx: &PropositionDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#propositionField.
	fn visit_propositionfield(&mut self, ctx: &PropositionFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#subjectField.
	fn visit_subjectfield(&mut self, ctx: &SubjectFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#quantityField.
	fn visit_quantityfield(&mut self, ctx: &QuantityFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#unitField.
	fn visit_unitfield(&mut self, ctx: &UnitFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#scopeField.
	fn visit_scopefield(&mut self, ctx: &ScopeFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#queryField.
	fn visit_queryfield(&mut self, ctx: &QueryFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#resultField.
	fn visit_resultfield(&mut self, ctx: &ResultFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#argumentDecl.
	fn visit_argumentdecl(&mut self, ctx: &ArgumentDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#argumentFlow.
	fn visit_argumentflow(&mut self, ctx: &ArgumentFlowContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#support.
	fn visit_support(&mut self, ctx: &SupportContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#supportGroup.
	fn visit_supportgroup(&mut self, ctx: &SupportGroupContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#supportKind.
	fn visit_supportkind(&mut self, ctx: &SupportKindContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#patternDecl.
	fn visit_patterndecl(&mut self, ctx: &PatternDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#patternParameter.
	fn visit_patternparameter(&mut self, ctx: &PatternParameterContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#parameterKind.
	fn visit_parameterkind(&mut self, ctx: &ParameterKindContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#applicationDecl.
	fn visit_applicationdecl(&mut self, ctx: &ApplicationDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#patternBinding.
	fn visit_patternbinding(&mut self, ctx: &PatternBindingContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#bindingValue.
	fn visit_bindingvalue(&mut self, ctx: &BindingValueContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#patternGuard.
	fn visit_patternguard(&mut self, ctx: &PatternGuardContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#objectionDecl.
	fn visit_objectiondecl(&mut self, ctx: &ObjectionDeclContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#objectionSupport.
	fn visit_objectionsupport(&mut self, ctx: &ObjectionSupportContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#objectionGroup.
	fn visit_objectiongroup(&mut self, ctx: &ObjectionGroupContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#objectionSupportKind.
	fn visit_objectionsupportkind(&mut self, ctx: &ObjectionSupportKindContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#targetKind.
	fn visit_targetkind(&mut self, ctx: &TargetKindContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#argumentationDirective.
	fn visit_argumentationdirective(&mut self, ctx: &ArgumentationDirectiveContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#referenceList.
	fn visit_referencelist(&mut self, ctx: &ReferenceListContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#reference.
	fn visit_reference(&mut self, ctx: &ReferenceContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#qualifiedName.
	fn visit_qualifiedname(&mut self, ctx: &QualifiedNameContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#predicate.
	fn visit_predicate(&mut self, ctx: &PredicateContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#expression.
	fn visit_expression(&mut self, ctx: &ExpressionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#orExpression.
	fn visit_orexpression(&mut self, ctx: &OrExpressionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#andExpression.
	fn visit_andexpression(&mut self, ctx: &AndExpressionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#comparisonExpression.
	fn visit_comparisonexpression(&mut self, ctx: &ComparisonExpressionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#additiveExpression.
	fn visit_additiveexpression(&mut self, ctx: &AdditiveExpressionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#multiplicativeExpression.
	fn visit_multiplicativeexpression(&mut self, ctx: &MultiplicativeExpressionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#unaryExpression.
	fn visit_unaryexpression(&mut self, ctx: &UnaryExpressionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#primaryExpression.
	fn visit_primaryexpression(&mut self, ctx: &PrimaryExpressionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#comparator.
	fn visit_comparator(&mut self, ctx: &ComparatorContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#jsonValue.
	fn visit_jsonvalue(&mut self, ctx: &JsonValueContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#jsonObject.
	fn visit_jsonobject(&mut self, ctx: &JsonObjectContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#jsonArray.
	fn visit_jsonarray(&mut self, ctx: &JsonArrayContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#jsonScalar.
	fn visit_jsonscalar(&mut self, ctx: &JsonScalarContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#signedNumber.
	fn visit_signednumber(&mut self, ctx: &SignedNumberContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#identifier.
	fn visit_identifier(&mut self, ctx: &IdentifierContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by EALParser#lineEnd.
	fn visit_lineend(&mut self, ctx: &LineEndContext<'input>) {
            self.visit_children(ctx)
        }

}