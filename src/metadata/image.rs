use crate::{
    metadata::{class::*, file_reader::*, metadata::*, method::*, signature::*, token::*},
    util::{holder::*, name_path::*},
};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

pub type RVA = u32;

#[derive(Debug, Clone)]
pub struct Image {
    /// CLI Info
    pub cli_info: CLIInfo,

    /// Metadata streams
    pub metadata: MetaDataStreams,

    /// PE file reader
    pub reader: Option<Rc<RefCell<PEFileReader>>>,

    /// Cache ``MethodInfoRef`` by RVA
    pub method_cache: FxHashMap<RVA, MethodInfoRef>,

    /// Cache ``ClassInfoRef`` by token
    pub class_cache: FxHashMap<Token, ClassInfoRef>,

    /// Holds all the standard classes like ``System::Object``. Will be removed in the future.
    standard_classes: TypeHolder<ClassInfoRef>,
    // pub memberref_cache: FxHashMap<usize, MethodBodyRef>
}

impl Image {
    pub fn new(
        cli_info: CLIInfo,
        metadata: MetaDataStreams,
        reader: Option<Rc<RefCell<PEFileReader>>>,
    ) -> Self {
        Self {
            cli_info,
            metadata,
            reader,
            method_cache: FxHashMap::default(),
            class_cache: FxHashMap::default(),
            standard_classes: TypeHolder::new(),
        }
    }

    pub fn setup_all_class(&mut self) {
        self.setup_all_standard_class();

        let typedefs = &self.metadata.metadata_stream.tables[TableKind::TypeDef.into_num()];
        let typerefs = &self.metadata.metadata_stream.tables[TableKind::TypeRef.into_num()];
        let fields = &self.metadata.metadata_stream.tables[TableKind::Field.into_num()];
        let methoddefs = &self.metadata.metadata_stream.tables[TableKind::MethodDef.into_num()];
        let mut methods_to_setup = vec![];
        let mut fields_to_setup = vec![];
        let mut extends = vec![];

        for (i, typeref) in typerefs.iter().enumerate() {
            let token = encode_token(TableKind::TypeRef.into(), i as u32 + 1);
            let tref = retrieve!(typeref, Table::TypeRef);
            let namespace = self.get_string(tref.type_namespace).as_str();
            let name = self.get_string(tref.type_name).as_str();
            if let Some(class) = self
                .standard_classes
                .get(TypeFullPath("mscorlib", namespace, name))
            {
                self.class_cache.insert(token, class.clone());
            }
        }

        for (i, typedef) in typedefs.iter().enumerate() {
            let typedef = retrieve!(typedef, Table::TypeDef);
            let next_typedef = typedefs.get(i + 1);
            let field_list_bgn = typedef.field_list as usize - 1;
            let method_list_bgn = typedef.method_list as usize - 1;
            let (field_list_end, method_list_end) =
                next_typedef.map_or((fields.len(), methoddefs.len()), |table| {
                    let td = retrieve!(table, Table::TypeDef);
                    (td.field_list as usize - 1, td.method_list as usize - 1)
                });
            let class_info = Rc::new(RefCell::new(ClassInfo {
                resolution_scope: ResolutionScope::None,
                parent: None,
                fields: vec![],
                methods: vec![],
                name: self.get_string(typedef.type_name).clone(),
                namespace: self.get_string(typedef.type_namespace).clone(),
                method_table: vec![],
            }));

            self.class_cache.insert(
                encode_token(TableKind::TypeDef.into(), i as u32 + 1),
                class_info.clone(),
            );
            methods_to_setup.push((class_info.clone(), method_list_bgn..method_list_end));
            fields_to_setup.push((class_info.clone(), field_list_bgn..field_list_end));

            if typedef.extends != 0 {
                extends.push((class_info, typedef.extends))
            }
        }

        for (class, range) in fields_to_setup {
            let fields: Vec<ClassField> = fields[range]
                .iter()
                .map(|t| {
                    let ft = retrieve!(t, Table::Field);
                    let name = self.get_string(ft.name).clone();
                    let mut sig = self.get_blob(ft.signature).iter();
                    assert_eq!(sig.next().unwrap(), &0x06);
                    let ty = Type::into_type(self, &mut sig).unwrap();
                    ClassField { name, ty }
                })
                .collect();
            class.borrow_mut().fields = fields;
        }

        let file_reader_ref = self.reader.as_ref().unwrap().clone();
        let mut file_reader = file_reader_ref.borrow_mut();

        for (class, range) in methods_to_setup {
            let mut methods = vec![];

            for mdef in &methoddefs[range] {
                let mdef = retrieve!(mdef, Table::MethodDef);
                let method = file_reader
                    .read_method(self, class.clone(), mdef.rva)
                    .unwrap();
                self.method_cache.insert(mdef.rva, method.clone());
                methods.push(method)
            }

            class.borrow_mut().methods = methods;
        }

        for (class, extends) in extends {
            let token = decode_typedef_or_ref_token(extends as u32);
            let typedef_or_ref = self.get_table_entry(token);
            match typedef_or_ref {
                Table::TypeDef(_) => {
                    class.borrow_mut().parent =
                        Some(self.class_cache.get(&token.into()).unwrap().clone());
                }
                Table::TypeRef(_) => {} // TODO
                _ => unreachable!(),
            }
        }

        self.setup_all_class_method_table();
    }

    fn setup_all_class_method_table(&mut self) {
        for (_token, class_ref) in &self.class_cache {
            self.construct_class_method_table(class_ref);
        }
    }

    fn construct_class_method_table(&self, class_ref: &ClassInfoRef) {
        let mut class = class_ref.borrow_mut();
        let mut method_table = match &class.parent {
            Some(parent) => {
                self.construct_class_method_table(parent);
                parent.borrow().method_table.clone()
            }
            // If already borrowed, it means that ``class`` is System::Object.
            None => self
                .standard_classes
                .get(TypeFullPath("mscorlib", "System", "Object"))
                .unwrap()
                .try_borrow()
                .map(|sys_obj| sys_obj.methods.clone())
                .unwrap_or_else(|_| class.methods.clone()),
        };

        for minforef in &class.methods {
            let minfo = minforef.borrow();
            if let Some(m) = method_table
                .iter_mut()
                .find(|m| m.borrow().get_name() == minfo.get_name())
            {
                // Override
                *m = minforef.clone()
            } else {
                // New slot
                method_table.push(minforef.clone());
            }
        }

        class.method_table = method_table;
    }

    fn setup_all_standard_class(&mut self) {
        let class_system_object_ref = ClassInfo::new_ref(
            ResolutionScope::asm_ref("mscorlib"),
            "System",
            "Object",
            vec![],
            vec![],
            None,
        );
        let class_system_int32_ref = ClassInfo::new_ref(
            ResolutionScope::asm_ref("mscorlib"),
            "System",
            "Int32",
            vec![],
            vec![],
            Some(class_system_object_ref.clone()),
        );

        let system_object_to_string = Rc::new(RefCell::new(MethodInfo::MRef(MemberRefInfo {
            name: "ToString".to_string(),
            ty: Type::full_method_ty(/*this*/ 0x20, Type::string_ty(), &[]),
            class: class_system_object_ref.clone(),
        })));
        let system_int32_to_string = Rc::new(RefCell::new(MethodInfo::MRef(MemberRefInfo {
            name: "ToString".to_string(),
            ty: Type::full_method_ty(/*this*/ 0x20, Type::string_ty(), &[]),
            class: class_system_int32_ref.clone(),
        })));

        {
            let mut class_system_object = class_system_object_ref.borrow_mut();
            let mut class_system_int32 = class_system_int32_ref.borrow_mut();
            class_system_object
                .methods
                .push(system_object_to_string.clone());
            class_system_int32
                .methods
                .push(system_int32_to_string.clone());
        }

        self.standard_classes.add(
            TypeFullPath("mscorlib", "System", "Object"),
            class_system_object_ref,
        );
        self.standard_classes.add(
            TypeFullPath("mscorlib", "System", "Int32"),
            class_system_int32_ref,
        );
    }

    pub fn get_table_entry<T: Into<Token>>(&self, token: T) -> Table {
        let DecodedToken(table, entry) = decode_token(token.into());
        self.metadata.metadata_stream.tables[table as usize][entry as usize - 1]
    }

    pub fn get_string<T: Into<u32>>(&self, n: T) -> &String {
        self.metadata.strings.get(&n.into()).unwrap()
    }

    pub fn get_user_string<T: Into<u32>>(&self, n: T) -> &Vec<u16> {
        self.metadata.user_strings.get(&n.into()).unwrap()
    }

    pub fn get_entry_method(&mut self) -> MethodInfoRef {
        let method_or_file = self.get_table_entry(self.cli_info.cli_header.entry_point_token);
        let mdef = match method_or_file {
            Table::MethodDef(t) => t,
            // TOOD: File
            _ => unimplemented!(),
        };
        self.get_method_by_rva(mdef.rva)
    }

    pub fn get_method_by_rva(&self, rva: u32) -> MethodInfoRef {
        self.method_cache.get(&rva).unwrap().clone()
    }

    pub fn get_method_def_table_by_rva(&self, rva: u32) -> Option<&MethodDefTable> {
        for method_def in &self.metadata.metadata_stream.tables[TableKind::MethodDef.into_num()] {
            match method_def {
                Table::MethodDef(mdt) if mdt.rva == rva => return Some(mdt),
                Table::MethodDef(_) => continue,
                _ => return None,
            }
        }
        None
    }

    pub fn get_blob<T: Into<u32>>(&self, n: T) -> &Vec<u8> {
        self.metadata.blob.get(&n.into()).unwrap()
    }

    pub fn get_info_from_type_ref_table(
        &self,
        type_ref_table: &TypeRefTable,
    ) -> (&String, &String, &String) {
        let token = type_ref_table.resolution_scope_table_and_entry();
        let assembly_ref_table = retrieve!(self.get_table_entry(token), Table::AssemblyRef);
        let asm_ref_name = self.get_string(assembly_ref_table.name);
        let ty_namespace = self.get_string(type_ref_table.type_namespace);
        let ty_name = self.get_string(type_ref_table.type_name);
        (asm_ref_name, ty_namespace, ty_name)
    }

    pub fn get_method_ref_type_from_signature(&self, signature: u16) -> Type {
        let sig = self.get_blob(signature);
        SignatureParser::new(sig)
            .parse_method_ref_sig(self)
            .unwrap()
    }
}
